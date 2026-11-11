use super::{Provider, http_agent, read_json};

const OLLAMA_DEFAULT: &str = "http://127.0.0.1:11434";
const LM_STUDIO_DEFAULT: &str = "http://127.0.0.1:1234";

/// Base URL of a locally hosted runtime. Ollama already documents `OLLAMA_HOST`
/// for people who moved it off the default port, so honour that rather than
/// inventing a second setting.
pub fn base_url(provider: Provider) -> String {
    let (variable, default) = match provider {
        Provider::LmStudio => ("LMSTUDIO_HOST", LM_STUDIO_DEFAULT),
        _ => ("OLLAMA_HOST", OLLAMA_DEFAULT),
    };
    std::env::var(variable).map_or_else(|_| default.to_owned(), |value| normalize(&value))
}

fn normalize(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_owned();
    }
    format!("http://{trimmed}")
}

/// Model ids the runtime currently has pulled. Empty means it is not running.
pub fn models(provider: Provider) -> Vec<String> {
    let base = base_url(provider);
    let Ok(mut response) = http_agent().get(format!("{base}/v1/models")).call() else {
        return Vec::new();
    };
    let Ok(value) = read_json(provider, &mut response) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["id"].as_str())
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .collect();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn bare_host_gains_a_scheme() {
        assert_eq!(normalize("localhost:11434"), "http://localhost:11434");
    }

    #[test]
    fn explicit_scheme_and_trailing_slash_are_preserved_and_trimmed() {
        assert_eq!(normalize("https://box.lan:1234/"), "https://box.lan:1234");
    }
}
