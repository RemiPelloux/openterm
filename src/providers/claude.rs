use super::{AskRequest, Provider, api_key, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};

const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const MAX_OUTPUT_TOKENS: u16 = 1_024;

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let body = json!({
        "model": model(request)?,
        "max_tokens": MAX_OUTPUT_TOKENS,
        "messages": [{"role": "user", "content": request.prompt}],
    });
    let mut response = http_agent()
        .post(ENDPOINT)
        .header("x-api-key", api_key("ANTHROPIC_API_KEY")?)
        .header("anthropic-version", API_VERSION)
        .send_json(body)
        .map_err(|error| http_error(Provider::Claude, &error))?;
    let value = read_json(Provider::Claude, &mut response)?;
    parse(&value)
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
