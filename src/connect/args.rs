//! `ai-connect` argument parsing. API keys are only ever read from stdin.

use crate::providers::Provider;
use std::ffi::OsString;

const KEY_ON_ARGV: &str = "API keys are never accepted as arguments; pipe the key with --key-stdin";
const KEY_FLAGS: [&str; 2] = ["--key", "--api-key"];

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Help,
    Interactive,
    Status { json: bool },
    SetDefault(Provider),
    Connect(ConnectArgs),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConnectArgs {
    pub provider: Provider,
    pub model: Option<String>,
    pub key_stdin: bool,
    pub base_url: Option<String>,
    pub set_default: bool,
}

#[derive(Default)]
struct Raw {
    provider: Option<String>,
    options: Options,
    default: Option<String>,
    status: bool,
    json: bool,
}

/// Flags that only make sense next to a PROVIDER.
#[derive(Default, PartialEq, Eq)]
struct Options {
    model: Option<String>,
    key_stdin: bool,
    base_url: Option<String>,
    no_default: bool,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
    let mut raw = Raw::default();
    let mut args = args
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned());
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(Action::Help);
        }
        raw.take(&arg, &mut args)?;
    }
    raw.finish()
}

impl Raw {
    fn take(&mut self, arg: &str, rest: &mut impl Iterator<Item = String>) -> Result<(), String> {
        match arg {
            "--status" => self.status = true,
            "--json" => self.json = true,
            "--key-stdin" => self.options.key_stdin = true,
            "--no-default" => self.options.no_default = true,
            "--model" => self.options.model = Some(value(arg, rest)?),
            "--base-url" => self.options.base_url = Some(value(arg, rest)?),
            "--default" => self.default = Some(value(arg, rest)?),
            flag if is_key_flag(flag) => return Err(KEY_ON_ARGV.to_owned()),
            flag if flag.starts_with('-') => return Err(format!("unknown option '{flag}'")),
            name if self.provider.is_none() => self.provider = Some(name.to_owned()),
            extra => return Err(format!("unexpected argument '{extra}'")),
        }
        Ok(())
    }

    fn finish(self) -> Result<Action, String> {
        if self.json && !self.status {
            return Err("--json requires --status".to_owned());
        }
        if self.status {
            return self
                .only("--status")
                .map(|()| Action::Status { json: self.json });
        }
        if let Some(name) = &self.default {
            self.only("--default")?;
            return Ok(Action::SetDefault(name.parse()?));
        }
        let Some(name) = &self.provider else {
            return self.only("PROVIDER").map(|()| Action::Interactive);
        };
        Ok(Action::Connect(ConnectArgs {
            provider: name.parse()?,
            model: self.options.model,
            key_stdin: self.options.key_stdin,
            base_url: self.options.base_url,
            set_default: !self.options.no_default,
        }))
    }

    /// Rejects connect flags next to a mode that does not use them.
    fn only(&self, mode: &str) -> Result<(), String> {
        let with_default = self.default.is_some() && mode != "--default";
        let extra = self.provider.is_some() || self.options != Options::default() || with_default;
        if extra {
            return Err(format!("{mode} cannot be combined with connect options"));
        }
        Ok(())
    }
}

fn is_key_flag(flag: &str) -> bool {
    KEY_FLAGS.iter().any(|key| {
        flag == *key
            || flag
                .strip_prefix(key)
                .is_some_and(|rest| rest.starts_with('='))
    })
}

fn value(flag: &str, rest: &mut impl Iterator<Item = String>) -> Result<String, String> {
    rest.next()
        .filter(|value| !value.is_empty() && !value.starts_with('-'))
        .ok_or_else(|| format!("{flag} requires a value"))
}

#[cfg(test)]
mod tests {
    use super::{Action, ConnectArgs, parse};
    use crate::providers::{Compat, Provider};
    use std::ffi::OsString;

    fn run(args: &[&str]) -> Result<Action, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn no_args_is_the_wizard_and_status_takes_json() {
        assert_eq!(run(&[]), Ok(Action::Interactive));
        assert_eq!(run(&["--status"]), Ok(Action::Status { json: false }));
        assert_eq!(
            run(&["--status", "--json"]),
            Ok(Action::Status { json: true })
        );
        assert!(run(&["--json"]).is_err());
    }

    #[test]
    fn positional_provider_with_flags() {
        let action = run(&["groq", "--model", "m", "--key-stdin", "--no-default"]);
        let expected = ConnectArgs {
            provider: Provider::Compat(Compat::Groq),
            model: Some("m".to_owned()),
            key_stdin: true,
            base_url: None,
            set_default: false,
        };
        assert_eq!(action, Ok(Action::Connect(expected)));
    }

    #[test]
    fn rejects_keys_on_the_command_line() {
        for args in [&["groq", "--key", "sk-x"][..], &["groq", "--api-key=sk-x"]] {
            let error = run(args).expect_err("key flag");
            assert!(error.contains("--key-stdin"));
            assert!(!error.contains("sk-x"));
        }
    }

    #[test]
    fn default_switches_provider_only() {
        assert_eq!(
            run(&["--default", "ollama"]),
            Ok(Action::SetDefault(Provider::Ollama))
        );
        assert!(run(&["--default", "ollama", "groq"]).is_err());
        assert!(run(&["--status", "groq"]).is_err());
    }

    #[test]
    fn rejects_bad_usage() {
        assert!(run(&["nope"]).is_err());
        assert!(run(&["groq", "extra"]).is_err());
        assert!(run(&["groq", "--model"]).is_err());
        assert!(run(&["--model", "m"]).is_err());
        assert!(run(&["--bogus"]).is_err());
    }
}
