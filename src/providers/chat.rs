//! OpenAI-compatible Chat Completions, shared by Mistral, local runtimes, and
//! every gateway preset.

use super::{AskRequest, Provider, http_agent, model, non_empty, read_json};
use serde_json::{Value, json};

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let body = json!({
        "model": model(request)?,
        "messages": [
            {"role": "system", "content": super::ask_prompt::INSTRUCTIONS},
            {"role": "user", "content": request.prompt},
        ],
        "stream": false,
    });
    let mut call = http_agent().post(request.endpoint.url("chat/completions"));
    if request.provider.uses_api_key() {
        let key = request.endpoint.api_key(request.provider)?;
        call = call.header("Authorization", format!("Bearer {key}"));
    }
    let mut response = call
        .send_json(body)
        .map_err(|error| request_error(request, &error))?;
    let value = read_json(request.provider, &mut response)?;
    super::openai::report_usage(&value);
    parse(request.provider, &value)
}

fn request_error(request: &AskRequest, error: &ureq::Error) -> String {
    let provider = request.provider;
    if provider.is_local() {
        let base = &request.endpoint.base_url;
        return format!("{provider} is not reachable at {base} ({error}); start it and retry");
    }
    super::http_error(provider, error)
}

pub(super) fn parse(provider: Provider, value: &Value) -> Result<String, String> {
    let text = value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    non_empty(provider, text)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::providers::{Compat, Provider};
    use serde_json::json;

    #[test]
    fn parses_chat_completion_text() {
        let value = json!({"choices": [{"message": {"content": "Use lsof -i."}}]});
        assert_eq!(
            parse(Provider::Ollama, &value).as_deref(),
            Ok("Use lsof -i.")
        );
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(
            parse(Provider::Compat(Compat::Groq), &json!({})).unwrap_err(),
            "groq returned no text"
        );
    }
}
