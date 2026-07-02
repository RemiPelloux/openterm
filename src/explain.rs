use crate::providers::{AskRequest, Provider};
use std::ffi::OsString;

const HELP: &str =
    "Usage: ai-explain [--last|--result REF] [--provider PROVIDER] [--model MODEL] [QUESTION]

Explain stored command output or the latest result using your selected model.
Read-only — does not modify files or execute commands.

Providers: codex (default), openai, claude, mistral
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let parsed = match parse(
        args.into_iter()
            .map(|arg| arg.to_string_lossy().to_string()),
    ) {
        Ok(Some(request)) => request,
        Ok(None) => return print_help(),
        Err(error) => return print_error(&error, 2),
    };
    match parsed.provider.ask(&parsed) {
        Ok(answer) => {
            println!("{answer}");
            0
        }
        Err(error) => print_error(&error, 1),
    }
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<AskRequest>, String> {
    let mut provider = Provider::Codex;
    let mut model = None;
    let mut use_last = false;
    let mut result = None;
    let mut question = Vec::new();
    let mut args = args.peekable();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(None),
            "--provider" => provider = parse_provider(args.next())?,
            "--model" => model = Some(required_value("--model", args.next())?),
            "--last" => use_last = true,
            "--result" => result = Some(required_value("--result", args.next())?),
            value if value.starts_with('-') => return Err(format!("unknown option '{value}'")),
            value => question.push(value.to_owned()),
        }
    }

    build_request(ExplainOptions {
        provider,
        model,
        use_last,
        result,
        question,
    })
}

struct ExplainOptions {
    provider: Provider,
    model: Option<String>,
    use_last: bool,
    result: Option<String>,
    question: Vec<String>,
}

fn build_request(options: ExplainOptions) -> Result<Option<AskRequest>, String> {
    let model = crate::providers::resolve_model(options.provider, options.model)?;
    let context = result_context(options.use_last, options.result)?;
    let focus = if options.question.is_empty() {
        "Explain this command output clearly and concisely. Highlight errors, next steps, and what matters for an agent.".to_owned()
    } else {
        options.question.join(" ")
    };
    let prompt = format!("{context}\n\nQuestion:\n{focus}");
    Ok(Some(AskRequest {
        provider: options.provider,
        model,
        prompt,
    }))
}

fn result_context(use_last: bool, result: Option<String>) -> Result<String, String> {
    if use_last && result.is_some() {
        return Err("use either --last or --result, not both".to_owned());
    }
    match result {
        Some(id) => crate::agent_context::result_block(&id),
        None => crate::agent_context::latest_result_block(),
    }
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

    #[test]
    fn rejects_both_last_and_result() {
        let result = parse(
            ["--last", "--result", "res_abc", "why failed"]
                .map(str::to_owned)
                .into_iter(),
        );
        assert!(result.unwrap_err().contains("not both"));
    }

    #[test]
    fn requires_stored_result_by_default() {
        let result = parse(["summarize this"].map(str::to_owned).into_iter());
        assert!(result.unwrap_err().contains("no stored results yet"));
    }
}
