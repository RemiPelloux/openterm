//! Live model discovery against each provider's own listing endpoint.

use crate::providers::{Endpoint, Provider, http_agent, read_json};
use serde_json::Value;

const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Anthropic pages `/v1/models` (default 20); ask for its maximum page.
const ANTHROPIC_PAGE_LIMIT: &str = "1000";

/// Model ids sorted newest first. `None` when nothing could be listed, so
/// callers keep whatever they already cached.
pub fn fetch(provider: Provider, endpoint: &Endpoint) -> Option<Vec<String>> {
    if provider.is_cli() {
        return None;
    }
    if provider.is_local() {
        // Only the models actually pulled locally can answer, so never guess.
        let models = crate::providers::local_models(provider);
        return (!models.is_empty()).then_some(models);
    }
    if endpoint.base_url.is_empty() {
        return None;
    }
    fetch_hosted(provider, endpoint)
}

/// One thread per provider: discovery is network-bound, so N providers cost
/// roughly one round-trip instead of N.
pub fn fetch_all(targets: &[(Provider, Endpoint)]) -> Vec<Option<Vec<String>>> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = targets
            .iter()
            .map(|(provider, endpoint)| scope.spawn(move || fetch(*provider, endpoint)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().ok().flatten())
            .collect()
    })
}

fn fetch_hosted(provider: Provider, endpoint: &Endpoint) -> Option<Vec<String>> {
    let key = endpoint.optional_api_key().unwrap_or_default();
    let request = match provider {
        Provider::Claude => http_agent()
            .get(endpoint.url("v1/models"))
            .query("limit", ANTHROPIC_PAGE_LIMIT)
            .header("x-api-key", key)
            .header("anthropic-version", ANTHROPIC_VERSION),
        Provider::OpenAi => http_agent()
            .get(endpoint.url("v1/models"))
            .header("Authorization", format!("Bearer {key}")),
        _ => http_agent()
            .get(endpoint.url("models"))
            .header("Authorization", format!("Bearer {key}")),
    };
    let mut response = request.call().ok()?;
    let value = read_json(provider, &mut response).ok()?;
    Some(sort_models(&value, created_field(provider)))
}

fn created_field(provider: Provider) -> &'static str {
    if provider == Provider::Claude {
        "created_at"
    } else {
        "created"
    }
}

fn sort_models(value: &Value, created_field: &str) -> Vec<String> {
    let mut ranked = Vec::new();
    for item in value["data"].as_array().into_iter().flatten() {
        let id = item["id"].as_str().unwrap_or("").trim();
        if id.is_empty() {
            continue;
        }
        ranked.push((created_rank(&item[created_field]), id.to_owned()));
    }
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    ranked.into_iter().map(|(_, id)| id).collect()
}

/// Unix seconds for OpenAI-style listings; Anthropic sends RFC 3339 strings,
/// which already sort chronologically as text.
fn created_rank(value: &Value) -> (i64, String) {
    let number = json_i64(value).unwrap_or(0);
    let text = value.as_str().unwrap_or_default().to_owned();
    (number, text)
}

fn json_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
}

#[cfg(test)]
mod tests {
    use super::{fetch, json_i64, sort_models};
    use crate::providers::{Compat, Endpoint, Provider};
    use serde_json::json;

    #[test]
    fn api_models_sort_newest_first() {
        let value = json!({
            "data": [
                {"id": "old", "created": 1},
                {"id": "new", "created": 99},
                {"id": "  "}
            ]
        });
        assert_eq!(sort_models(&value, "created"), ["new", "old"]);
    }

    #[test]
    fn anthropic_timestamps_sort_newest_first() {
        let value = json!({
            "data": [
                {"id": "claude-a", "created_at": "2024-10-22T00:00:00Z"},
                {"id": "claude-b", "created_at": "2025-05-14T00:00:00Z"}
            ]
        });
        assert_eq!(sort_models(&value, "created_at"), ["claude-b", "claude-a"]);
    }

    #[test]
    fn ties_sort_by_id() {
        let value = json!({"data": [{"id": "b"}, {"id": "a"}]});
        assert_eq!(sort_models(&value, "created"), ["a", "b"]);
    }

    #[test]
    fn json_i64_reads_unsigned() {
        assert_eq!(json_i64(&json!(42)), Some(42));
    }

    #[test]
    fn cli_and_unset_custom_skip_the_network() {
        assert_eq!(fetch(Provider::Codex, &Endpoint::default()), None);
        let custom = Provider::Compat(Compat::Custom);
        assert_eq!(fetch(custom, &Endpoint::default()), None);
    }
}
