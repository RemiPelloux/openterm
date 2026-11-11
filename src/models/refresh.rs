//! `ai-models --refresh`: fetch every target concurrently, then one cache write.

use super::cache::{self, CachedModels};
use super::discover;
use crate::providers::config::ProviderSettings;
use crate::providers::{Endpoint, Provider};

/// Exit status: 0 when every target refreshed, 1 otherwise.
pub fn run(settings: &ProviderSettings, only: Option<Provider>, quiet: bool) -> i32 {
    let targets = targets(settings, only);
    if targets.is_empty() {
        if !quiet {
            eprintln!("ai-models: nothing to refresh (no connected provider with model discovery)");
        }
        return i32::from(only.is_some());
    }
    let results = discover::fetch_all(&targets);
    let (updates, failed) = collect(targets, results, quiet);
    let saved = updates.is_empty() || save(updates, quiet);
    i32::from(failed || !saved)
}

type Update = (Provider, CachedModels);

fn collect(
    targets: Vec<(Provider, Endpoint)>,
    results: Vec<Option<Vec<String>>>,
    quiet: bool,
) -> (Vec<Update>, bool) {
    let now = cache::now_ms();
    let mut updates = Vec::new();
    let mut failed = false;
    for ((provider, endpoint), models) in targets.into_iter().zip(results) {
        let Some(models) = models else {
            failed = true;
            if !quiet {
                eprintln!("{provider}: discovery failed; kept the cached list");
            }
            continue;
        };
        if !quiet {
            eprintln!("{provider}: {} models", models.len());
        }
        updates.push((provider, CachedModels::new(&endpoint, models, now)));
    }
    (updates, failed)
}

/// The named provider when it has a discovery endpoint, else every
/// connected one that does.
fn targets(settings: &ProviderSettings, only: Option<Provider>) -> Vec<(Provider, Endpoint)> {
    let candidates: Vec<Provider> = match only {
        Some(provider) if !provider.is_cli() => vec![provider],
        Some(_) => Vec::new(),
        None => Provider::ALL
            .into_iter()
            .filter(|provider| has_discovery(*provider, settings))
            .collect(),
    };
    candidates
        .into_iter()
        .map(|provider| (provider, Endpoint::resolve(provider, settings)))
        .collect()
}

/// Usable without further setup and able to list its models: a key and an
/// endpoint for hosted APIs, a chosen model for local runtimes.
pub fn has_discovery(provider: Provider, settings: &ProviderSettings) -> bool {
    if provider.is_cli() {
        return false;
    }
    if provider.is_local() {
        return settings.is_stored(provider);
    }
    settings.api_key(provider).is_some()
        && !Endpoint::resolve(provider, settings).base_url.is_empty()
}

fn save(updates: Vec<Update>, quiet: bool) -> bool {
    match cache::store(updates) {
        Ok(()) => true,
        Err(error) => {
            if !quiet {
                eprintln!("ai-models: could not write the model cache: {error}");
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{has_discovery, targets};
    use crate::providers::config::{ProviderEntry, ProviderSettings};
    use crate::providers::{Compat, Provider};

    #[test]
    fn only_connected_discoverable_providers_are_targets() {
        let mut settings = ProviderSettings::default();
        let groq = Provider::Compat(Compat::Groq);
        let custom = Provider::Compat(Compat::Custom);
        for provider in [groq, custom] {
            let entry = ProviderEntry {
                api_key: "k".to_owned(),
                ..ProviderEntry::default()
            };
            settings.set_entry(provider, entry);
        }
        assert!(has_discovery(groq, &settings));
        assert!(!has_discovery(Provider::Codex, &settings));
        assert!(!has_discovery(Provider::Ollama, &settings));
        let named = targets(&settings, Some(Provider::OpenAi));
        assert_eq!(named.len(), 1);
        assert!(targets(&settings, Some(Provider::ClaudeCode)).is_empty());
    }
}
