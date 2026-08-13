use super::{AskRequest, Provider, api_key, http_agent, http_error, model, read_json};
use serde_json::{Value, json};

const ENDPOINT: &str = "https://api.mistral.ai/v1/chat/completions";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let body = json!({
        "model": model(request)?,
        "messages": [
            {"role": "system", "content": super::ask_prompt::INSTRUCTIONS},
            {"role": "user", "content": request.prompt},
        ],
    });
    let mut response = http_agent()
        .post(ENDPOINT)
        .header(
            "Authorization",
            format!("Bearer {}", api_key("MISTRAL_API_KEY")?),
        )
        .send_json(body)
        .map_err(|error| http_error(Provider::Mistral, &error))?;
    let value = read_json(Provider::Mistral, &mut response)?;
    parse(&value)
}

fn parse(value: &Value) -> Result<String, String> {
    value["choices"][0]["message"]["content"]
        .as_str()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "mistral returned no text".to_owned())
}

#[cfg(test)]
mod tests {
    use super::parse;
    use serde_json::json;

    #[test]
    fn parses_chat_completion_text() {
        let value = json!({"choices": [{"message": {"content": "Use ps aux."}}]});
        assert_eq!(parse(&value).as_deref(), Ok("Use ps aux."));
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(parse(&json!({})).unwrap_err(), "mistral returned no text");
    }
}
