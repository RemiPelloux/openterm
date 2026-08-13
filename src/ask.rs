use crate::providers::{AskRequest, Provider};
use std::ffi::OsString;

const HELP: &str =
    "Usage: ai-ask [--provider PROVIDER] [--model MODEL] [--workspace] [--result REF] <QUESTION>

Providers: openai, claude, mistral, ollama, lmstudio, or codex (fallback)

Modes:
  --workspace   Include read-only cwd and Git context
  --result REF  Include bounded preview of a stored result

API providers require --model (or run ai-connect) and a configured API key.
Local providers (ollama, lmstudio) require a running runtime and --model
(or run ai-connect). They take no API key.
ai-ask gives advice only; it does not modify files or execute proposed commands.
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let parsed = parse(
        args.into_iter()
            .map(|arg| arg.to_string_lossy().to_string()),
    );
    let request = match parsed {
        Ok(Some(request)) => request,
        Ok(None) => return print_help(),
        Err(error) => return print_error(&error, 2),
    };

    match crate::ask_cache::complete(&request, || request.provider.ask(&request)) {
        Ok(answer) => {
            println!("{answer}");
            0
        }
        Err(error) => print_error(&error, 1),
    }
}

pub fn ask_workspace(question: &str) -> Result<String, String> {
    let request = parse(["--workspace".to_owned(), question.to_owned()].into_iter())?
        .ok_or_else(|| "a question is required".to_owned())?;
    crate::ask_cache::complete(&request, || request.provider.ask(&request))
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<AskRequest>, String> {
    let mut provider = crate::providers::config::default_provider();
    let mut model = None;
    let mut workspace = false;
    let mut result = None;
    let mut prompt = Vec::new();
    let mut args = args.peekable();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(None),
            "--provider" => provider = parse_provider(args.next())?,
            "--model" => model = Some(required_value("--model", args.next())?),
            "--workspace" => workspace = true,
            "--result" => result = Some(required_value("--result", args.next())?),
            value if value.starts_with('-') => return Err(format!("unknown option '{value}'")),
            value => prompt.push(value.to_owned()),
        }
    }

    build_request(AskOptions {
        provider,
        model,
        workspace,
        result,
        prompt,
    })
}

struct AskOptions {
    provider: Provider,
    model: Option<String>,
    workspace: bool,
    result: Option<String>,
    prompt: Vec<String>,
}

fn build_request(options: AskOptions) -> Result<Option<AskRequest>, String> {
    if options.prompt.is_empty() {
        return Err("a question is required".to_owned());
    }
    let model = crate::providers::resolve_model(options.provider, options.model)?;
    let question = options.prompt.join(" ");
    let full_prompt = attach_context(options.workspace, options.result, &question)?;
    Ok(Some(AskRequest {
        provider: options.provider,
        model,
        prompt: full_prompt,
    }))
}

fn attach_context(
    workspace: bool,
    result: Option<String>,
    question: &str,
) -> Result<String, String> {
    let mut parts = Vec::new();
    if workspace {
        parts.push(crate::agent_context::workspace_block());
    }
    if let Some(id) = result {
        parts.push(crate::agent_context::result_block(&id)?);
    }
    if parts.is_empty() {
        return Ok(question.to_owned());
    }
    parts.push(format!("Question:\n{question}"));
    Ok(parts.join("\n\n"))
}

fn parse_provider(value: Option<String>) -> Result<Provider, String> {
    required_value("--provider", value)?.parse()
}

fn required_value(option: &str, value: Option<String>) -> Result<String, String> {
    value.ok_or_else(|| format!("{option} requires a value"))
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
    use super::parse;
    use crate::providers::Provider;

    #[test]
    fn provider_flag_selects_codex() {
        let request = parse(
            ["--provider", "codex", "show disk usage"]
                .map(str::to_owned)
                .into_iter(),
        )
        .expect("valid args")
        .expect("request");
        assert_eq!(request.provider, Provider::Codex);
    }

    #[test]
    fn workspace_mode_adds_context_header() {
        let request = parse(
            ["--workspace", "what changed?"]
                .map(str::to_owned)
                .into_iter(),
        )
        .expect("valid args")
        .expect("request");
        assert!(request.prompt.contains("OpenTerm workspace context"));
        assert!(request.prompt.contains("Question:"));
    }

    #[test]
    fn explicit_model_is_kept() {
        let request = parse(
            ["--provider", "openai", "--model", "gpt-test", "hello"]
                .map(str::to_owned)
                .into_iter(),
        )
        .expect("valid args")
        .expect("request");
        assert_eq!(request.provider, Provider::OpenAi);
        assert_eq!(request.model.as_deref(), Some("gpt-test"));
    }

    #[test]
    fn unknown_option_is_rejected() {
        let result = parse(["--write", "hello"].map(str::to_owned).into_iter());
        assert_eq!(result.unwrap_err(), "unknown option '--write'");
    }

    #[test]
    fn unknown_provider_is_rejected() {
        let result = parse(
            ["--provider", "other", "hello"]
                .map(str::to_owned)
                .into_iter(),
        );
        assert_eq!(result.unwrap_err(), "unknown provider 'other'");
    }

    #[test]
    fn local_provider_accepts_explicit_model() {
        let request = parse(
            ["--provider", "ollama", "--model", "llama3.2", "hello"]
                .map(str::to_owned)
                .into_iter(),
        )
        .expect("valid args")
        .expect("request");
        assert_eq!(request.provider, Provider::Ollama);
        assert_eq!(request.model.as_deref(), Some("llama3.2"));
    }
}
