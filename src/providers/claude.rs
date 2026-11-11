use super::anthropic_events::MessageStream;
use super::sse::{self, Response};
use super::{AskRequest, Provider, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};

pub(crate) const API_VERSION: &str = "2023-06-01";
const MAX_OUTPUT_TOKENS: u16 = 4_096;

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let mut response = send(request, false)?;
    parse(&read_json(Provider::Claude, &mut response)?)
}

pub fn ask_stream(request: &AskRequest, on_delta: &mut dyn FnMut(&str)) -> Result<String, String> {
    let mut response = send(request, true)?;
    if !sse::is_event_stream(&response) {
        let text = parse(&read_json(Provider::Claude, &mut response)?)?;
        on_delta(&text);
        return Ok(text);
    }
    let mut stream = MessageStream::new(Provider::Claude, on_delta);
    sse::read_events(Provider::Claude, sse::body(&mut response), &mut |event| {
        stream.take(event)
    })?;
    stream.usage().record();
    stream.finish()
}

fn send(request: &AskRequest, stream: bool) -> Result<Response, String> {
    let body = json!({
        "model": model(request)?,
        "max_tokens": MAX_OUTPUT_TOKENS,
        "system": super::ask_prompt::INSTRUCTIONS,
        "messages": [{"role": "user", "content": request.prompt}],
        "stream": stream,
    });
    http_agent()
        .post(request.endpoint.url("v1/messages"))
        .header("x-api-key", request.endpoint.api_key(Provider::Claude)?)
        .header("anthropic-version", API_VERSION)
        .send_json(body)
        .map_err(|error| http_error(Provider::Claude, &error))
}

fn parse(value: &Value) -> Result<String, String> {
    let text = value["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|content| content["text"].as_str())
        .collect::<Vec<_>>()
        .join("");
    non_empty(Provider::Claude, text)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use serde_json::json;

    #[test]
    fn parses_message_text() {
        let value = json!({"content": [{"type": "text", "text": "Use du -sh *."}]});
        assert_eq!(parse(&value).as_deref(), Ok("Use du -sh *."));
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(parse(&json!({})).unwrap_err(), "claude returned no text");
    }
}
