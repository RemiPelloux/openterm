//! `ai-models --json` contract (consumed by the macOS GUI) and the human view.
//! Field names are a public contract: never rename them, and never add keys.

use super::cache::ModelCache;
use crate::providers::config::ProviderSettings;
use crate::providers::{Compat, Endpoint, Provider};
use serde::Serialize;
use std::fmt::Write;

const MS_PER_MINUTE: u64 = 60 * 1000;
const MINUTES_PER_HOUR: u64 = 60;
const HOURS_PER_DAY: u64 = 24;

#[derive(Debug, Serialize)]
pub struct Report {
    pub default_provider: String,
    pub providers: Vec<ProviderReport>,
}

#[derive(Debug, Serialize)]
pub struct ProviderReport {
    pub id: String,
    pub name: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub connected: bool,
    pub default: bool,
    pub model: Option<String>,
    pub base_url: String,
    pub needs_base_url: bool,
    pub models: Vec<String>,
    pub fetched_at_ms: Option<u64>,
}

/// Every provider in `Provider::ALL` order; `connected` is index-aligned.
pub fn build(settings: &ProviderSettings, cache: &ModelCache, connected: &[bool]) -> Report {
    let default = settings.default_provider();
    let providers = Provider::ALL
        .into_iter()
        .zip(connected)
        .map(|(provider, &connected)| entry(provider, settings, cache, connected))
        .collect();
    Report {
        default_provider: default.to_string(),
        providers,
    }
}

fn entry(
    provider: Provider,
    settings: &ProviderSettings,
    cache: &ModelCache,
    connected: bool,
) -> ProviderReport {
    let base_url = Endpoint::resolve(provider, settings).base_url;
    let cached = cache.for_endpoint(provider, &base_url);
    ProviderReport {
        id: provider.to_string(),
        name: provider.short_label(),
        label: provider.label(),
        kind: provider.kind(),
        connected,
        default: provider == settings.default_provider(),
        model: settings.model(provider).map(str::to_owned),
        needs_base_url: provider == Provider::Compat(Compat::Custom),
        models: cached.map(|entry| entry.models.clone()).unwrap_or_default(),
        fetched_at_ms: cached.map(|entry| entry.fetched_at_ms),
        base_url,
    }
}

/// Connected providers (or just `only`), each with its cached models and the
/// stored model marked `*`.
pub fn render_table(report: &Report, only: Option<Provider>, now_ms: u64) -> String {
    let mut out = String::new();
    for item in report.providers.iter().filter(|item| shown(item, only)) {
        out.push_str(&heading(item, now_ms));
        out.push_str(&model_lines(item));
    }
    if out.is_empty() {
        return "No connected providers. Run ai-connect first.\n".to_owned();
    }
    out
}

fn shown(item: &ProviderReport, only: Option<Provider>) -> bool {
    match only {
        Some(provider) => item.id == provider.to_string(),
        None => item.connected,
    }
}

fn heading(item: &ProviderReport, now_ms: u64) -> String {
    let default = if item.default { " (default)" } else { "" };
    let age = match item.fetched_at_ms {
        _ if item.kind == "cli" => String::new(),
        Some(fetched) => format!(" · fetched {}", age(now_ms.saturating_sub(fetched))),
        None => " · not fetched".to_owned(),
    };
    format!("{}{default} · {}{age}\n", item.id, item.label)
}

fn model_lines(item: &ProviderReport) -> String {
    if item.kind == "cli" {
        return "  the CLI chooses its own model\n".to_owned();
    }
    if item.models.is_empty() {
        return format!("  no cached models — run ai-models {} --refresh\n", item.id);
    }
    let mut lines = String::new();
    for model in &item.models {
        let mark = if item.model.as_ref() == Some(model) {
            '*'
        } else {
            ' '
        };
        let _ = writeln!(lines, "  {mark} {model}");
    }
    lines
}

fn age(elapsed_ms: u64) -> String {
    let minutes = elapsed_ms / MS_PER_MINUTE;
    let hours = minutes / MINUTES_PER_HOUR;
    if minutes == 0 {
        "just now".to_owned()
    } else if hours == 0 {
        format!("{minutes}m ago")
    } else if hours < HOURS_PER_DAY {
        format!("{hours}h ago")
    } else {
        format!("{}d ago", hours / HOURS_PER_DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::{age, build, render_table};
    use crate::models::cache::{CachedModels, ModelCache};
    use crate::providers::config::{ProviderEntry, ProviderSettings};
    use crate::providers::{Compat, Provider};
    use serde_json::Value;

    const GROQ: Provider = Provider::Compat(Compat::Groq);
    const FETCHED_AT: u64 = 1_731_300_000_000;
    const GROQ_INDEX: usize = 8;
    const CONTRACT_KEYS: [&str; 11] = [
        "id",
        "name",
        "label",
        "kind",
        "connected",
        "default",
        "model",
        "base_url",
        "needs_base_url",
        "models",
        "fetched_at_ms",
    ];

    fn sorted(keys: [&str; 11]) -> Vec<&str> {
        let mut keys = keys.to_vec();
        keys.sort_unstable();
        keys
    }

    fn sorted_keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    fn fixture() -> (ProviderSettings, ModelCache) {
        let mut settings = ProviderSettings {
            default_provider: Some("claude-code".to_owned()),
            ..ProviderSettings::default()
        };
        let entry = ProviderEntry {
            api_key: "sk-secret".to_owned(),
            model: Some("llama-3.3-70b-versatile".to_owned()),
            base_url: None,
        };
        settings.set_entry(GROQ, entry);
        let mut cache = ModelCache::default();
        let cached = CachedModels {
            base_url: "https://api.groq.com/openai/v1".to_owned(),
            fetched_at_ms: FETCHED_AT,
            models: vec!["llama-3.3-70b-versatile".to_owned(), "other".to_owned()],
        };
        cache.insert(GROQ, cached);
        (settings, cache)
    }

    fn connected() -> Vec<bool> {
        Provider::ALL
            .iter()
            .map(|provider| *provider == GROQ)
            .collect()
    }

    #[test]
    fn json_matches_the_gui_contract() {
        let (settings, cache) = fixture();
        let json = serde_json::to_value(build(&settings, &cache, &connected())).expect("json");
        assert_eq!(json["default_provider"], "claude-code");
        let providers = json["providers"].as_array().expect("providers");
        assert_eq!(providers.len(), Provider::ALL.len());
        assert_eq!(providers[0]["id"], "codex");
        let groq = &providers[GROQ_INDEX];
        assert_eq!(sorted_keys(groq), sorted(CONTRACT_KEYS));
        assert_eq!(groq["id"], "groq");
        assert_eq!(groq["label"], "Groq");
        assert_eq!(groq["kind"], "api");
        assert_eq!(groq["connected"], true);
        assert_eq!(groq["base_url"], "https://api.groq.com/openai/v1");
        assert_eq!(groq["fetched_at_ms"], FETCHED_AT);
        assert!(!json.to_string().contains("sk-secret"));
    }

    #[test]
    fn json_nulls_kinds_and_custom_flag() {
        let (settings, cache) = fixture();
        let json = serde_json::to_value(build(&settings, &cache, &connected())).expect("json");
        let providers = json["providers"].as_array().expect("providers");
        let by_id = |id: &str| providers.iter().find(|item| item["id"] == id).expect(id);
        let cli = by_id("claude-code");
        assert_eq!(
            (cli["kind"].clone(), cli["default"].clone()),
            ("cli".into(), true.into())
        );
        assert_eq!(cli["base_url"], "");
        assert_eq!(cli["model"], Value::Null);
        assert_eq!(cli["fetched_at_ms"], Value::Null);
        assert_eq!(by_id("ollama")["kind"], "local");
        assert_eq!(by_id("custom")["needs_base_url"], true);
        assert_eq!(by_id("openai")["needs_base_url"], false);
    }

    #[test]
    fn table_marks_the_stored_model() {
        let (settings, cache) = fixture();
        let report = build(&settings, &cache, &connected());
        let text = render_table(&report, None, FETCHED_AT);
        assert!(text.starts_with("groq · Groq · fetched just now\n"));
        assert!(text.contains("  * llama-3.3-70b-versatile\n    other\n"));
        let named = render_table(&report, Some(Provider::OpenAi), FETCHED_AT);
        assert!(named.contains("no cached models — run ai-models openai --refresh"));
    }

    #[test]
    fn ages_are_compact() {
        assert_eq!(age(0), "just now");
        assert_eq!(age(5 * 60 * 1000), "5m ago");
        assert_eq!(age(3 * 60 * 60 * 1000), "3h ago");
        assert_eq!(age(50 * 60 * 60 * 1000), "2d ago");
    }
}
