use super::{AskRequest, Provider, api_key, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};

const ENDPOINT: &str = "https://api.openai.com/v1/responses";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let api_key = api_key("OPENAI_API_KEY")?;
    let body = json!({ "model": model(request)?, "input": request.prompt });
    let mut response = http_agent()
        .post(ENDPOINT)
        .header("Authorization", format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|error| http_error(Provider::OpenAi, &error))?;
    let value = read_json(Provider::OpenAi, &mut response)?;
    parse(&value)
}

fn parse(value: &Value) -> Result<String, String> {
    let text = value["output"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|item| item["content"].as_array().into_iter().flatten())
        .filter_map(|content| content["text"].as_str())
        .collect::<Vec<_>>()
        .join("");
    non_empty(Provider::OpenAi, text)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use serde_json::json;

    #[test]
    fn parses_responses_api_text() {
        let value =
            json!({"output": [{"content": [{"type": "output_text", "text": "Use df -h."}]}]});
        assert_eq!(parse(&value).as_deref(), Ok("Use df -h."));
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(parse(&json!({})).unwrap_err(), "openai returned no text");
    }
}
