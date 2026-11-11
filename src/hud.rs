use crate::providers::{Endpoint, Provider, config};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const JSON_NAME: &str = "hud.json";
const LINE_NAME: &str = "hud";

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hud {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    #[serde(default)]
    pub tokens_cached: u64,
    #[serde(default)]
    pub cache_hits: u64,
    #[serde(default)]
    pub live: bool,
}

/// Token counters only; provider/model fields are synced by `ai-connect`, so
/// the per-request hot path never re-reads provider settings.
pub fn record_usage(input: u64, output: u64, cached: u64) {
    let mut hud = load();
    hud.tokens_in = hud.tokens_in.saturating_add(input);
    hud.tokens_out = hud.tokens_out.saturating_add(output);
    hud.tokens_cached = hud.tokens_cached.saturating_add(cached);
    hud.live = true;
    save(&hud);
}

pub fn record_cache_hit() {
    let mut hud = load();
    hud.cache_hits = hud.cache_hits.saturating_add(1);
    save(&hud);
}

pub fn sync_from_config(settings: &config::ProviderSettings) {
    let mut hud = load();
    sync_fields(&mut hud, settings);
    save(&hud);
}

pub fn current_line() -> String {
    format_line(&load())
}

pub fn format_line(hud: &Hud) -> String {
    if hud.provider.is_empty() {
        return "ai · not connected".to_owned();
    }
    let model = if hud.model.is_empty() {
        String::new()
    } else {
        format!(" {}", hud.model)
    };
    let state = if hud.live { "live" } else { "idle" };
    format!(
        "{}{model} · {} tok · {} cached · {state}",
        hud.provider,
        compact(hud.tokens_in),
        compact(hud.tokens_cached)
    )
}

pub fn compact(value: u64) -> String {
    if value < 1_000 {
        return value.to_string();
    }
    if value < 1_000_000 {
        let tenths = value / 100;
        return format!("{}.{}k", tenths / 10, tenths % 10);
    }
    let tenths = value / 100_000;
    format!("{}.{}M", tenths / 10, tenths % 10)
}

fn sync_fields(hud: &mut Hud, settings: &config::ProviderSettings) {
    if settings.default_provider.is_none() {
        return;
    }
    let provider = settings.default_provider();
    hud.provider = provider.to_string();
    settings
        .model(provider)
        .unwrap_or_default()
        .clone_into(&mut hud.model);
    hud.host = host_label(provider, settings);
}

fn host_label(provider: Provider, settings: &config::ProviderSettings) -> String {
    let endpoint = Endpoint::resolve(provider, settings);
    if endpoint.base_url.is_empty() {
        return provider.to_string();
    }
    strip_scheme(&endpoint.base_url)
}

fn strip_scheme(url: &str) -> String {
    url.trim()
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned()
}

fn load() -> Hud {
    hud_json_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save(hud: &Hud) {
    let Some(dir) = data_dir() else {
        return;
    };
    let json = serde_json::to_string_pretty(hud).unwrap_or_else(|_| "{}".to_owned());
    let _ = std::fs::write(dir.join(JSON_NAME), format!("{json}\n"));
    let _ = std::fs::write(dir.join(LINE_NAME), format!("{}\n", format_line(hud)));
}

fn data_dir() -> Option<PathBuf> {
    let dir = crate::results::data_root().ok()?;
    crate::results::create_private_directory(&dir).ok()?;
    Some(dir)
}

fn hud_json_path() -> Option<PathBuf> {
    Some(data_dir()?.join(JSON_NAME))
}

#[cfg(test)]
mod tests {
    use super::{Hud, compact, format_line, strip_scheme};

    #[test]
    fn compact_token_counts() {
        assert_eq!(compact(42), "42");
        assert_eq!(compact(12_400), "12.4k");
        assert_eq!(compact(2_000_000), "2.0M");
    }

    #[test]
    fn line_shows_idle_and_live() {
        let idle = Hud {
            provider: "openai".to_owned(),
            model: "gpt-5.6-terra".to_owned(),
            ..Hud::default()
        };
        assert_eq!(
            format_line(&idle),
            "openai gpt-5.6-terra · 0 tok · 0 cached · idle"
        );
        let live = Hud {
            live: true,
            tokens_in: 12_400,
            tokens_cached: 8_100,
            ..idle
        };
        assert_eq!(
            format_line(&live),
            "openai gpt-5.6-terra · 12.4k tok · 8.1k cached · live"
        );
        assert_eq!(format_line(&Hud::default()), "ai · not connected");
    }

    #[test]
    fn strip_scheme_keeps_host() {
        assert_eq!(
            strip_scheme("https://gateway.example/"),
            "gateway.example"
        );
    }
}
