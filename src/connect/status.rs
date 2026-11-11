//! `ai-connect --status [--json]`: connection state and key source per
//! provider. Never prints a key.

use crate::providers::Provider;
use crate::providers::config::{self, KeySource, ProviderSettings};
use serde_json::{Value, json};

struct Row {
    provider: Provider,
    connected: bool,
    source: &'static str,
    model: Option<String>,
}

pub fn print(json: bool) {
    let settings = config::load();
    let rows: Vec<Row> = Provider::ALL
        .into_iter()
        .map(|provider| row(provider, &settings))
        .collect();
    if json {
        println!("{}", to_json(&settings, &rows));
    } else {
        print_text(&settings, &rows);
    }
    crate::hud::sync_from_config(&settings);
}

fn row(provider: Provider, settings: &ProviderSettings) -> Row {
    let (connected, source) = if provider.is_cli() {
        let ready = super::cli_session_ready(provider);
        (ready, if ready { "cli" } else { "none" })
    } else if provider.is_local() {
        let ready = settings.is_stored(provider);
        (ready, if ready { "local" } else { "none" })
    } else {
        let source = settings.key_source(provider);
        (source != KeySource::None, source.as_str())
    };
    let model = settings.model(provider).map(str::to_owned);
    Row {
        provider,
        connected,
        source,
        model,
    }
}

fn to_json(settings: &ProviderSettings, rows: &[Row]) -> Value {
    let providers: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.provider.to_string(),
                "label": row.provider.label(),
                "connected": row.connected,
                "source": row.source,
                "model": row.model,
            })
        })
        .collect();
    json!({
        "default_provider": settings.default_provider().to_string(),
        "providers": providers,
    })
}

fn print_text(settings: &ProviderSettings, rows: &[Row]) {
    println!("OpenTerm provider status\n");
    for row in rows {
        println!("  {:<12} {}", row.provider, describe(row));
    }
    if let Ok(path) = config::config_path() {
        println!("\nConfig: {}", path.display());
    }
    if let Some(default) = settings.default_provider.as_deref() {
        println!("Default provider: {default}");
    }
    println!("\nRun ai-connect to connect with arrow-key selection.");
}

fn describe(row: &Row) -> String {
    let provider = row.provider;
    let model = row
        .model
        .as_deref()
        .map(|value| format!(", model {value}"))
        .unwrap_or_default();
    let host = crate::providers::local_base_url(provider);
    match (row.connected, row.source) {
        (true, "cli") => format!("connected ({provider} CLI session)"),
        (true, "local") => format!("connected (local at {host}{model})"),
        (true, "env") => format!("configured (environment{model})"),
        (true, "keychain") => format!("connected (keychain{model})"),
        (true, _) => format!("connected (providers.json{model})"),
        _ if provider.is_cli() => format!("not connected — run {}", super::login_hint(provider)),
        _ if provider.is_local() => {
            format!("not connected — start {provider} at {host}, then run ai-connect")
        }
        _ => "not connected — run ai-connect".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Row, to_json};
    use crate::providers::Provider;
    use crate::providers::config::ProviderSettings;

    #[test]
    fn json_has_the_documented_shape() {
        let settings = ProviderSettings {
            default_provider: Some("ollama".to_owned()),
            ..ProviderSettings::default()
        };
        let rows = [Row {
            provider: Provider::Ollama,
            connected: true,
            source: "local",
            model: Some("llama3.2".to_owned()),
        }];
        let value = to_json(&settings, &rows);
        assert_eq!(value["default_provider"], "ollama");
        let entry = &value["providers"][0];
        assert_eq!(entry["id"], "ollama");
        assert_eq!(entry["label"], Provider::Ollama.label());
        assert_eq!(entry["connected"], true);
        assert_eq!(entry["source"], "local");
        assert_eq!(entry["model"], "llama3.2");
    }

    #[test]
    fn json_model_is_null_when_unset() {
        let rows = [Row {
            provider: Provider::Codex,
            connected: false,
            source: "none",
            model: None,
        }];
        let value = to_json(&ProviderSettings::default(), &rows);
        assert_eq!(value["default_provider"], "codex");
        assert!(value["providers"][0]["model"].is_null());
    }
}
