use crate::providers::{Provider, http_agent, read_json};
use serde_json::Value;

const CODEX_MODELS_URL: &str =
    "https://chatgpt.com/backend-api/codex/models?client_version=1.0.0";

const CODEX_FALLBACK: &[&str] = &[
    "gpt-5.5",
    "gpt-5.4-mini",
    "gpt-5.4",
    "gpt-5.3-codex",
    "gpt-5.3-codex-spark",
];

pub fn list_for_provider(provider: Provider, api_key: &str) -> Vec<String> {
    let models = match provider {
        Provider::OpenAi => fetch_openai(api_key),
        Provider::Claude => fetch_claude(api_key),
        Provider::Mistral => fetch_mistral(api_key),
        Provider::Codex => Vec::new(),
    };
    if models.is_empty() {
        fallback_for(provider)
    } else {
        models
    }
}

pub fn list_codex(access_token: &str) -> Vec<String> {
    let models = fetch_codex(access_token);
    if models.is_empty() {
        CODEX_FALLBACK.iter().map(|model| (*model).to_owned()).collect()
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

fn fetch_codex(access_token: &str) -> Vec<String> {
    let Ok(mut response) = http_agent()
        .get(CODEX_MODELS_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .call()
    else {
        return Vec::new();
    };
    read_json(Provider::Codex, &mut response)
        .ok()
        .map(|value| parse_codex_models(&value))
        .unwrap_or_default()
}

fn parse_codex_models(value: &Value) -> Vec<String> {
    let mut ranked = Vec::new();
    for item in value["models"].as_array().into_iter().flatten() {
        let slug = item["slug"].as_str().unwrap_or("").trim();
        if slug.is_empty() {
            continue;
        }
        let visibility = item["visibility"].as_str().unwrap_or("").to_ascii_lowercase();
        if visibility == "hide" || visibility == "hidden" {
            continue;
        }
        let priority = json_i64(&item["priority"]).unwrap_or(10_000);
        ranked.push((priority, slug.to_owned()));
    }
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    ranked.into_iter().map(|(_, slug)| slug).collect()
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
    value.as_i64().or_else(|| {
        value
            .as_u64()
            .and_then(|number| i64::try_from(number).ok())
    })
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
        Provider::Codex => CODEX_FALLBACK.iter().map(|model| (*model).to_owned()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{json_i64, parse_codex_models, sort_models};
    use serde_json::json;

    #[test]
    fn codex_models_sort_by_priority() {
        let value = json!({
            "models": [
                {"slug": "gpt-5.3-codex", "priority": 2},
                {"slug": "gpt-5.5", "priority": 1, "visibility": "visible"},
                {"slug": "hidden-model", "priority": 0, "visibility": "hide"}
            ]
        });
        assert_eq!(
            parse_codex_models(&value),
            vec!["gpt-5.5".to_owned(), "gpt-5.3-codex".to_owned()]
        );
    }

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
