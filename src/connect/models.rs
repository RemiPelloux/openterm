use crate::providers::{Provider, http_agent, read_json};
use serde_json::Value;

pub fn list_for_provider(provider: Provider, api_key: &str) -> Vec<String> {
    let models = match provider {
        Provider::OpenAi => fetch_openai(api_key),
        Provider::Claude => fetch_claude(api_key),
        Provider::Mistral => fetch_mistral(api_key),
        Provider::Ollama | Provider::LmStudio => crate::providers::local_models(provider),
        Provider::Codex => Vec::new(),
    };
    if provider.is_local() {
        // Only the models actually pulled locally can answer, so never guess.
        return models;
    }
    if models.is_empty() {
        fallback_for(provider)
    } else {
        models
    }
}

fn fetch_openai(api_key: &str) -> Vec<String> {
    let Ok(mut response) = http_agent()
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .call()
    else {
        return Vec::new();
    };
    read_json(Provider::OpenAi, &mut response)
        .ok()
        .map(|value| sort_models(&value, "created"))
        .unwrap_or_default()
}

fn fetch_claude(api_key: &str) -> Vec<String> {
    let Ok(mut response) = http_agent()
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .call()
    else {
        return Vec::new();
    };
    read_json(Provider::Claude, &mut response)
        .ok()
        .map(|value| sort_models(&value, "created_at"))
        .unwrap_or_default()
}

fn fetch_mistral(api_key: &str) -> Vec<String> {
    let Ok(mut response) = http_agent()
        .get("https://api.mistral.ai/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .call()
    else {
        return Vec::new();
    };
    read_json(Provider::Mistral, &mut response)
        .ok()
        .map(|value| sort_models(&value, "created"))
        .unwrap_or_default()
}

fn sort_models(value: &Value, created_field: &str) -> Vec<String> {
    let mut ranked = Vec::new();
    for item in value["data"].as_array().into_iter().flatten() {
        let id = item["id"].as_str().unwrap_or("").trim();
        if id.is_empty() {
            continue;
        }
        let created = json_i64(&item[created_field]).unwrap_or(0);
        ranked.push((created, id.to_owned()));
    }
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    ranked.into_iter().map(|(_, id)| id).collect()
}

fn json_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
}

fn fallback_for(provider: Provider) -> Vec<String> {
    match provider {
        Provider::OpenAi => vec![
            "gpt-4.1".into(),
            "gpt-4.1-mini".into(),
            "gpt-4o".into(),
            "gpt-4o-mini".into(),
        ],
        Provider::Claude => vec![
            "claude-sonnet-4-20250514".into(),
            "claude-3-7-sonnet-20250219".into(),
            "claude-3-5-haiku-20241022".into(),
        ],
        Provider::Mistral => vec![
            "mistral-large-latest".into(),
            "mistral-small-latest".into(),
            "codestral-latest".into(),
        ],
        Provider::Codex | Provider::Ollama | Provider::LmStudio => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{json_i64, sort_models};
    use serde_json::json;

    #[test]
    fn api_models_sort_newest_first() {
        let value = json!({
            "data": [
                {"id": "old", "created": 1},
                {"id": "new", "created": 99}
            ]
        });
        assert_eq!(
            sort_models(&value, "created"),
            vec!["new".to_owned(), "old".to_owned()]
        );
    }

    #[test]
    fn json_i64_reads_unsigned() {
        assert_eq!(json_i64(&json!(42)), Some(42));
    }
}
