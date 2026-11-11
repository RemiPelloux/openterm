use super::{AskRequest, Provider, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};

use super::ask_prompt::INSTRUCTIONS;

pub(crate) const CACHE_KEY: &str = "openterm-ask-v3";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let api_key = request.endpoint.api_key(Provider::OpenAi)?;
    let body = json!({
        "model": model(request)?,
        "instructions": INSTRUCTIONS,
        "input": request.prompt,
        "prompt_cache_key": CACHE_KEY,
    });
    let mut response = http_agent()
        .post(request.endpoint.url("v1/responses"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|error| http_error(Provider::OpenAi, &error))?;
    let value = read_json(Provider::OpenAi, &mut response)?;
    report_usage(&value);
    parse_text(&value)
}

pub(crate) fn report_usage(value: &Value) {
    let cached = token_count(value, "cached_tokens");
    let input = token_count(value, "input_tokens").max(token_count(value, "prompt_tokens"));
    let output = token_count(value, "output_tokens").max(token_count(value, "completion_tokens"));
    crate::hud::record_usage(input, output, cached);
    if cached == 0 {
        return;
    }
    eprintln!("openterm: prompt-cache cached={cached} input={input}");
}

fn token_count(value: &Value, field: &str) -> u64 {
    value["usage"][field]
        .as_u64()
        .or_else(|| value["usage"]["input_tokens_details"][field].as_u64())
        .or_else(|| value["usage"]["prompt_tokens_details"][field].as_u64())
        .unwrap_or(0)
}

pub(crate) fn parse_text(value: &Value) -> Result<String, String> {
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
    use super::parse_text;
    use serde_json::json;

    #[test]
    fn parses_responses_api_text() {
        let value =
            json!({"output": [{"content": [{"type": "output_text", "text": "Use df -h."}]}]});
        assert_eq!(parse_text(&value).as_deref(), Ok("Use df -h."));
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(
            parse_text(&json!({})).unwrap_err(),
            "openai returned no text"
        );
    }

    #[test]
    fn reads_cached_token_count() {
        let value = json!({
            "usage": {
                "input_tokens": 4000,
                "prompt_tokens_details": {"cached_tokens": 3200}
            }
        });
        assert_eq!(super::token_count(&value, "cached_tokens"), 3200);
        assert_eq!(super::token_count(&value, "input_tokens"), 4000);
    }
}
