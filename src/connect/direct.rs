//! Prompt-free `ai-connect PROVIDER ...` for scripts and the macOS app.

use super::args::ConnectArgs;
use crate::providers::config::keychain::{KeyStore, Keychain};
use crate::providers::config::{self, ProviderEntry, ProviderSettings, Saved};
use crate::providers::{Compat, Provider};
use std::io::Read;

const CUSTOM: Provider = Provider::Compat(Compat::Custom);
const URL_SCHEMES: [&str; 2] = ["https://", "http://"];

pub fn run(args: &ConnectArgs) -> Result<String, String> {
    let key = if args.key_stdin {
        Some(read_stdin_key()?)
    } else {
        None
    };
    if args.provider.is_cli() {
        super::ensure_cli_ready(args.provider)?;
    }
    let mut settings = config::load();
    let model = apply(args, key, &mut settings, &Keychain)?;
    config::save(&settings)?;
    crate::hud::sync_from_config(&settings);
    Ok(model)
}

pub fn set_default(provider: Provider) -> Result<(), String> {
    let mut settings = config::load();
    settings.default_provider = Some(provider.to_string());
    config::save(&settings)?;
    crate::hud::sync_from_config(&settings);
    Ok(())
}

/// Warns on stderr when a key could not go to the keychain.
pub fn report_saved(provider: Provider, saved: &Saved) {
    if let Saved::File(reason) = saved {
        eprintln!(
            "warning: keychain unavailable ({reason}); {provider} key stored in providers.json (mode 600)"
        );
    }
}

/// Updates `settings` in memory; returns the model label to print.
fn apply(
    args: &ConnectArgs,
    key: Option<String>,
    settings: &mut ProviderSettings,
    store: &dyn KeyStore,
) -> Result<String, String> {
    let provider = args.provider;
    check_flags(args, key.is_some())?;
    let mut entry = settings.entry(provider).cloned().unwrap_or_default();
    if let Some(url) = &args.base_url {
        entry.base_url = Some(normalize_url(url)?);
    }
    if provider == CUSTOM
        && Compat::Custom
            .base_url(entry.base_url.as_deref())
            .is_empty()
    {
        return Err("custom requires --base-url URL (e.g. https://host/v1)".to_owned());
    }
    entry.model = args.model.clone().or(entry.model);
    let model = model_label(provider, entry.model.as_deref())?;
    if entry != ProviderEntry::default() {
        settings.set_entry(provider, entry);
    }
    save_key(settings, provider, key, store)?;
    if args.set_default {
        settings.default_provider = Some(provider.to_string());
    }
    Ok(model)
}

fn check_flags(args: &ConnectArgs, has_key: bool) -> Result<(), String> {
    let provider = args.provider;
    if has_key && !provider.uses_api_key() {
        return Err(format!("{provider} takes no API key; drop --key-stdin"));
    }
    if args.base_url.is_some() && (provider.is_cli() || provider.is_local()) {
        return Err(format!("--base-url is not supported for {provider}"));
    }
    Ok(())
}

fn model_label(provider: Provider, model: Option<&str>) -> Result<String, String> {
    match model {
        Some(model) => Ok(model.to_owned()),
        None if provider.requires_model() => Err(format!(
            "no model chosen for {provider}; pass --model MODEL (list them with 'ai-models {provider}')"
        )),
        None => Ok(format!("{provider} CLI default model")),
    }
}

/// A piped key, else the legacy file key (migrated), else one already in
/// the environment or keychain.
fn save_key(
    settings: &mut ProviderSettings,
    provider: Provider,
    key: Option<String>,
    store: &dyn KeyStore,
) -> Result<(), String> {
    if !provider.uses_api_key() {
        return Ok(());
    }
    if let Some(key) = key.or_else(|| settings.file_key(provider)) {
        report_saved(provider, &settings.store_key(provider, key, store));
        return Ok(());
    }
    if settings.resolve_key(provider, store).is_some() {
        return Ok(());
    }
    let variable = config::env_var(provider).unwrap_or("its API key variable");
    Err(format!(
        "no API key for {provider}; pipe one with --key-stdin or set {variable}"
    ))
}

fn normalize_url(url: &str) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/');
    if URL_SCHEMES.iter().any(|scheme| url.starts_with(scheme)) {
        return Ok(url.to_owned());
    }
    Err(format!(
        "--base-url must start with https:// or http://, got '{url}'"
    ))
}

fn read_stdin_key() -> Result<String, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| format!("could not read the API key from stdin: {error}"))?;
    let key = text.trim_end_matches(['\n', '\r']);
    if key.trim().is_empty() {
        return Err("--key-stdin got an empty key".to_owned());
    }
    Ok(key.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{CUSTOM, apply};
    use crate::connect::args::ConnectArgs;
    use crate::providers::config::keychain::memory::MemoryStore;
    use crate::providers::config::{ProviderEntry, ProviderSettings};
    use crate::providers::{Compat, Provider};

    const GROQ: Provider = Provider::Compat(Compat::Groq);

    fn args(provider: Provider, model: Option<&str>) -> ConnectArgs {
        ConnectArgs {
            provider,
            model: model.map(str::to_owned),
            key_stdin: false,
            base_url: None,
            set_default: true,
        }
    }

    fn connect(
        args: &ConnectArgs,
        key: Option<&str>,
        store: &MemoryStore,
    ) -> Result<String, String> {
        let mut settings = ProviderSettings::default();
        apply(args, key.map(str::to_owned), &mut settings, store)
    }

    #[test]
    fn custom_requires_a_base_url() {
        let store = MemoryStore::default();
        let error = connect(&args(CUSTOM, Some("m")), Some("k"), &store).expect_err("no url");
        assert!(error.contains("--base-url"));
        let mut with_url = args(CUSTOM, Some("m"));
        with_url.base_url = Some("https://llm.example/v1/".to_owned());
        let mut settings = ProviderSettings::default();
        apply(&with_url, Some("k".into()), &mut settings, &store).expect("connect");
        assert_eq!(settings.base_url(CUSTOM), Some("https://llm.example/v1"));
    }

    #[test]
    fn piped_key_goes_to_keychain_and_sets_default() {
        let store = MemoryStore::default();
        let mut settings = ProviderSettings::default();
        let label = apply(
            &args(GROQ, Some("m")),
            Some("k".into()),
            &mut settings,
            &store,
        );
        assert_eq!(label.as_deref(), Ok("m"));
        assert_eq!(settings.file_key(GROQ), None);
        assert_eq!(
            store.items.borrow().get("groq").map(String::as_str),
            Some("k")
        );
        assert_eq!(settings.default_provider.as_deref(), Some("groq"));
    }

    #[test]
    fn reconnect_migrates_the_legacy_file_key() {
        let store = MemoryStore::default();
        let mut settings = ProviderSettings::default();
        let entry = ProviderEntry {
            api_key: "legacy".to_owned(),
            model: Some("m".to_owned()),
            base_url: None,
        };
        settings.set_entry(Provider::Mistral, entry);
        let mut no_default = args(Provider::Mistral, None);
        no_default.set_default = false;
        apply(&no_default, None, &mut settings, &store).expect("reconnect");
        assert_eq!(settings.file_key(Provider::Mistral), None);
        assert_eq!(
            store.items.borrow().get("mistral").map(String::as_str),
            Some("legacy")
        );
        assert_eq!(settings.default_provider, None);
    }

    #[test]
    fn missing_model_points_to_ai_models() {
        let error = connect(
            &args(Provider::Mistral, None),
            Some("k"),
            &MemoryStore::default(),
        );
        assert!(error.expect_err("model").contains("ai-models mistral"));
        let cli = connect(&args(Provider::Codex, None), None, &MemoryStore::default());
        assert_eq!(cli.as_deref(), Ok("codex CLI default model"));
    }

    #[test]
    fn rejects_keys_for_keyless_providers_and_missing_keys() {
        let store = MemoryStore::default();
        let local = connect(&args(Provider::Ollama, Some("llama")), Some("k"), &store);
        assert!(local.expect_err("keyless").contains("no API key"));
        let mut hosted = args(CUSTOM, Some("m"));
        hosted.base_url = Some("https://llm.example/v1".to_owned());
        let missing = connect(&hosted, None, &store);
        assert!(missing.expect_err("missing").contains("--key-stdin"));
    }
}
