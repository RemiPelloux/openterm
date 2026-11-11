use crate::models::cache::{self, CachedModels};
use crate::models::discover;
use crate::providers::{Endpoint, Provider};

/// Picker list for ai-connect: a fresh cached list when one exists, else a
/// live discovery that also refreshes the cache. Empty means discovery found
/// nothing, so the caller must ask for a model id instead of guessing one.
pub fn list_for_provider(provider: Provider, endpoint: &Endpoint) -> Vec<String> {
    if provider.is_cli() {
        return Vec::new();
    }
    if let Some(models) = fresh_cached(provider, endpoint) {
        return models;
    }
    let Some(models) = discover::fetch(provider, endpoint) else {
        return Vec::new();
    };
    let entry = CachedModels::new(endpoint, models.clone(), cache::now_ms());
    let _ = cache::store(vec![(provider, entry)]);
    models
}

/// Local runtimes are always asked live: only pulled models can answer.
fn fresh_cached(provider: Provider, endpoint: &Endpoint) -> Option<Vec<String>> {
    if provider.is_local() {
        return None;
    }
    cache::load()
        .for_endpoint(provider, &endpoint.base_url)
        .filter(|entry| entry.is_fresh(&endpoint.base_url, cache::now_ms()))
        .filter(|entry| !entry.models.is_empty())
        .map(|entry| entry.models.clone())
}
