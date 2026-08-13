use super::{AskRequest, Provider, api_key, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};

const DEFAULT_BASE: &str = "https://api.openai.com";
const CACHE_KEY: &str = "openterm-ask-v1";
const INSTRUCTIONS: &str = "You are OpenTerm's read-only terminal advisor. Give concise, actionable guidance. Do not modify files or execute commands. Prefer exact commands the user can run.";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let api_key = api_key("OPENAI_API_KEY")?;
    let body = json!({
        "model": model(request)?,
        "instructions": INSTRUCTIONS,
        "input": request.prompt,
        "prompt_cache_key": CACHE_KEY,
    });
    let mut response = http_agent()
        .post(url("v1/responses"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|error| http_error(Provider::OpenAi, &error))?;
    let value = read_json(Provider::OpenAi, &mut response)?;
    report_usage(&value);
    parse(&value)
}

pub(crate) fn url(path: &str) -> String {
    format!("{}/{path}", base())
}

fn base() -> String {
    resolve(
        std::env::var("OPENAI_BASE_URL").ok().as_deref(),
        super::config::stored_base_url(Provider::OpenAi).as_deref(),
    )
}

fn resolve(env: Option<&str>, stored: Option<&str>) -> String {
    env.filter(|value| !value.trim().is_empty())
        .or_else(|| stored.filter(|value| !value.trim().is_empty()))
        .map_or_else(|| DEFAULT_BASE.to_owned(), normalize)
}

fn normalize(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    match trimmed.strip_suffix("/v1") {
        Some(base) => base.trim_end_matches('/').to_owned(),
        None => trimmed.to_owned(),
    }
}

fn report_usage(value: &Value) {
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
    use super::{normalize, parse, resolve};
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

    #[test]
    fn resolve_prefers_env_then_stored_then_default() {
        assert_eq!(resolve(None, None), "https://api.openai.com");
        assert_eq!(
            resolve(None, Some("https://llm.example.com/v1/")),
            "https://llm.example.com"
        );
        assert_eq!(
            resolve(Some("https://gateway.example/"), Some("https://ignored")),
            "https://gateway.example"
        );
    }

    #[test]
    fn normalize_strips_slash_and_v1() {
        assert_eq!(
            normalize(" https://host.example/v1/ "),
            "https://host.example"
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
