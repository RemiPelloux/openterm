use super::ask_prompt::INSTRUCTIONS;
use super::sse::{self, Deltas, Response};
use super::{AskRequest, Provider, http_agent, http_error, model, read_json};
use serde_json::{Value, json};

pub fn ask_stream(request: &AskRequest, on_delta: &mut dyn FnMut(&str)) -> Result<String, String> {
    let mut response = start_stream(request)?;
    let mut deltas = Deltas::new(on_delta);
    if !sse::is_event_stream(&response) {
        return finish_json(&mut response, deltas);
    }
    sse::read_events(Provider::OpenAi, sse::body(&mut response), &mut |value| {
        take_event(value, &mut deltas);
        Ok(())
    })?;
    deltas.finish(Provider::OpenAi)
}

fn start_stream(request: &AskRequest) -> Result<Response, String> {
    let api_key = request.endpoint.api_key(Provider::OpenAi)?;
    let body = json!({
        "model": model(request)?,
        "instructions": INSTRUCTIONS,
        "input": request.prompt,
        "prompt_cache_key": super::openai::CACHE_KEY,
        "stream": true,
    });
    http_agent()
        .post(request.endpoint.url("v1/responses"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "text/event-stream")
        .send_json(body)
        .map_err(|error| http_error(Provider::OpenAi, &error))
}

fn finish_json(response: &mut Response, mut deltas: Deltas) -> Result<String, String> {
    let value = read_json(Provider::OpenAi, response)?;
    super::openai::report_usage(&value);
    deltas.push(&super::openai::parse_text(&value)?);
    deltas.finish(Provider::OpenAi)
}

fn take_event(value: &Value, deltas: &mut Deltas) {
    if let Some(delta) = delta_text(value) {
        deltas.push(delta);
    }
    if is_completed(value) {
        super::openai::report_usage(value.get("response").unwrap_or(value));
    }
}

fn delta_text(value: &Value) -> Option<&str> {
    let kind = value["type"].as_str().unwrap_or("");
    if kind.contains("output_text.delta") {
        return value["delta"]
            .as_str()
            .or_else(|| value["delta"]["text"].as_str());
    }
    value["choices"][0]["delta"]["content"].as_str()
}

fn is_completed(value: &Value) -> bool {
    value["type"].as_str() == Some("response.completed")
}

#[cfg(test)]
mod tests {
    use super::delta_text;
    use serde_json::json;

    #[test]
    fn parses_responses_delta_event() {
        let value = json!({"type": "response.output_text.delta", "delta": "Yes."});
        assert_eq!(delta_text(&value), Some("Yes."));
    }

    #[test]
    fn parses_chat_completion_delta() {
        let value = json!({"choices":[{"delta":{"content":"Hi"}}]});
        assert_eq!(delta_text(&value), Some("Hi"));
    }
}
