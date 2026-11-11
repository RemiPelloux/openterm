//! `ai-models`: cached, auto-refreshed model lists per provider.

mod auto;
pub mod cache;
mod choose;
pub mod discover;
mod refresh;
mod report;

pub use auto::refresh_if_stale;

use crate::providers::Provider;
use crate::providers::config::{self, ProviderSettings};
use std::ffi::OsString;

const HELP: &str = "Usage: ai-models [PROVIDER] [--refresh] [--json] [--quiet]
       ai-models use [PROVIDER [MODEL]]

List the models each connected provider offers, cached for 24 hours.
The stored model is marked with '*'.

  use         Choose the model (and default provider) with arrow keys,
              or directly: ai-models use openai gpt-6.1-sol
              Works for codex and claude-code too (e.g. ai-models use claude-code opus)

  PROVIDER    Only show (and refresh) this provider
  --refresh   Re-fetch from each provider's model discovery endpoint
  --json      Print every provider as JSON (for the macOS app)
  --quiet     Print nothing; used by the automatic background refresh

ai-ask and ai-explain refresh a stale list in the background.
";

#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    provider: Option<Provider>,
    refresh: bool,
    json: bool,
    quiet: bool,
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<OsString> = args.into_iter().collect();
    if args.first().is_some_and(|first| first == "use") {
        let rest: Vec<String> = args[1..]
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        return choose::run(&rest);
    }
    let options = match parse(args) {
        Ok(Some(options)) => options,
        Ok(None) => return print_help(),
        Err(error) => return print_error(&error, 2),
    };
    let settings = config::load();
    let status = if options.refresh {
        refresh::run(&settings, options.provider, options.quiet)
    } else {
        0
    };
    if !options.quiet {
        print_report(&settings, &options);
    }
    status
}

fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    for arg in args {
        match arg.to_string_lossy().as_ref() {
            "--help" | "-h" => return Ok(None),
            "--refresh" => options.refresh = true,
            "--json" => options.json = true,
            "--quiet" | "-q" => options.quiet = true,
            flag if flag.starts_with('-') => return Err(format!("unknown option '{flag}'")),
            _ if options.provider.is_some() => return Err("only one PROVIDER allowed".to_owned()),
            name => options.provider = Some(name.parse()?),
        }
    }
    Ok(Some(options))
}

fn print_report(settings: &ProviderSettings, options: &Options) {
    let built = report::build(settings, &cache::load(), &connected_flags(settings));
    if options.json {
        let json = serde_json::to_string(&built).unwrap_or_else(|_| "{}".to_owned());
        println!("{json}");
        return;
    }
    print!(
        "{}",
        report::render_table(&built, options.provider, cache::now_ms())
    );
}

/// Index-aligned with `Provider::ALL`. CLI login checks spawn processes, so
/// they run concurrently.
fn connected_flags(settings: &ProviderSettings) -> Vec<bool> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = Provider::ALL
            .into_iter()
            .map(|provider| scope.spawn(move || is_connected(provider, settings)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or(false))
            .collect()
    })
}

fn is_connected(provider: Provider, settings: &ProviderSettings) -> bool {
    if provider.is_cli() {
        return provider.cli_session_ready();
    }
    refresh::has_discovery(provider, settings)
}

fn print_help() -> i32 {
    print!("{HELP}");
    0
}

fn print_error(error: &str, code: i32) -> i32 {
    eprintln!("error: {error}\n\n{HELP}");
    code
}

#[cfg(test)]
mod tests {
    use super::{Options, parse};
    use crate::providers::{Compat, Provider};
    use std::ffi::OsString;

    fn parse_strs(args: &[&str]) -> Result<Option<Options>, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn parses_provider_and_flags() {
        let options = parse_strs(&["groq", "--refresh", "--quiet"])
            .expect("ok")
            .expect("run");
        assert_eq!(
            options,
            Options {
                provider: Some(Provider::Compat(Compat::Groq)),
                refresh: true,
                json: false,
                quiet: true,
            }
        );
    }

    #[test]
    fn rejects_unknown_input() {
        assert!(
            parse_strs(&["--nope"])
                .unwrap_err()
                .contains("unknown option")
        );
        assert!(
            parse_strs(&["nope"])
                .unwrap_err()
                .contains("unknown provider")
        );
        assert!(parse_strs(&["groq", "openai"]).is_err());
        assert!(parse_strs(&["--help"]).expect("ok").is_none());
    }
}
