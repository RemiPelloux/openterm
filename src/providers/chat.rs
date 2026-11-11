//! OpenAI-compatible Chat Completions, shared by Mistral, local runtimes, and
//! every gateway preset.

use super::sse::{self, Deltas, Response};
use super::{AskRequest, Compat, Provider, http_agent, model, non_empty, read_json};
use serde_json::{Value, json};

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let mut response = send(request, false)?;
    let value = read_json(request.provider, &mut response)?;
    super::openai::report_usage(&value);
    parse(request.provider, &value)
}

pub fn ask_stream(request: &AskRequest, on_delta: &mut dyn FnMut(&str)) -> Result<String, String> {
    let provider = request.provider;
    let mut response = send(request, true)?;
    let mut deltas = Deltas::new(on_delta);
    if !sse::is_event_stream(&response) {
        let value = read_json(provider, &mut response)?;
        super::openai::report_usage(&value);
        deltas.push(&parse(provider, &value)?);
        return deltas.finish(provider);
    }
    let mut usage = None;
    sse::read_events(provider, sse::body(&mut response), &mut |chunk| {
        take_chunk(provider, chunk, &mut deltas, &mut usage)
    })?;
    if let Some(chunk) = usage {
        super::openai::report_usage(&chunk);
    }
    deltas.finish(provider)
}

fn send(request: &AskRequest, stream: bool) -> Result<Response, String> {
    let mut call = http_agent().post(request.endpoint.url("chat/completions"));
    if request.provider.uses_api_key() {
        let key = request.endpoint.api_key(request.provider)?;
        call = call.header("Authorization", format!("Bearer {key}"));
    }
    call.send_json(body(request, stream)?)
        .map_err(|error| request_error(request, &error))
}

fn body(request: &AskRequest, stream: bool) -> Result<Value, String> {
    let mut body = json!({
        "model": model(request)?,
        "messages": [
            {"role": "system", "content": super::ask_prompt::INSTRUCTIONS},
            {"role": "user", "content": request.prompt},
        ],
        "stream": stream,
    });
    if stream && accepts_stream_options(request.provider) {
        body["stream_options"] = json!({"include_usage": true});
    }
    Ok(body)
}

/// Only gateways known to accept `stream_options`; Mistral, local runtimes,
/// and unknown custom servers may reject the unknown field.
const fn accepts_stream_options(provider: Provider) -> bool {
    matches!(
        provider,
        Provider::Compat(
            Compat::OpenRouter | Compat::Groq | Compat::DeepSeek | Compat::Xai | Compat::Together
        )
    )
}

/// Applies one streamed chunk; the chunk carrying `usage` is kept for the HUD.
fn take_chunk(
    provider: Provider,
    chunk: &Value,
    deltas: &mut Deltas,
    usage: &mut Option<Value>,
) -> Result<(), String> {
    if chunk.get("error").is_some() {
        return Err(sse::stream_error(provider, chunk));
    }
    if let Some(text) = chunk["choices"][0]["delta"]["content"].as_str() {
        deltas.push(text);
    }
    if chunk["usage"].is_object() {
        *usage = Some(chunk.clone());
    }
    Ok(())
}

fn request_error(request: &AskRequest, error: &ureq::Error) -> String {
    let provider = request.provider;
    if provider.is_local() {
        let base = &request.endpoint.base_url;
        return format!("{provider} is not reachable at {base} ({error}); start it and retry");
    }
    super::http_error(provider, error)
}

pub(super) fn parse(provider: Provider, value: &Value) -> Result<String, String> {
    let text = value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    non_empty(provider, text)
}

#[cfg(test)]
mod tests {
    use super::{accepts_stream_options, parse, take_chunk};
    use crate::providers::sse::{Deltas, read_events};
    use crate::providers::{Compat, Provider};
    use serde_json::{Value, json};

    fn stream(input: &str) -> (Result<String, String>, Option<Value>) {
        let mut on_delta = |_: &str| {};
        let mut deltas = Deltas::new(&mut on_delta);
        let mut usage = None;
        let read = read_events(Provider::Ollama, input.as_bytes(), &mut |chunk| {
            take_chunk(Provider::Ollama, chunk, &mut deltas, &mut usage)
        });
        (read.and_then(|()| deltas.finish(Provider::Ollama)), usage)
    }

    #[test]
    fn streams_chat_deltas_until_done() {
        let input = concat!(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Use \"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lsof.\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        assert_eq!(stream(input).0.as_deref(), Ok("Use lsof."));
    }

    #[test]
    fn keeps_final_usage_chunk() {
        let input = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
            "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":9,\"completion_tokens\":2}}\n\n",
            "data: [DONE]\n\n",
        );
        let usage = stream(input).1.expect("usage chunk");
        assert_eq!(usage["usage"]["prompt_tokens"], 9);
        assert_eq!(usage["usage"]["completion_tokens"], 2);
    }

    #[test]
    fn surfaces_stream_error() {
        let input = "data: {\"error\":{\"message\":\"rate limited\"}}\n\n";
        assert_eq!(
            stream(input).0.unwrap_err(),
            "ollama stream error: rate limited"
        );
    }

    #[test]
    fn asks_usage_only_from_known_gateways() {
        assert!(accepts_stream_options(Provider::Compat(Compat::OpenRouter)));
        assert!(!accepts_stream_options(Provider::Ollama));
        assert!(!accepts_stream_options(Provider::LmStudio));
        assert!(!accepts_stream_options(Provider::Mistral));
        assert!(!accepts_stream_options(Provider::Compat(Compat::Custom)));
    }

    #[test]
    fn parses_chat_completion_text() {
        let value = json!({"choices": [{"message": {"content": "Use lsof -i."}}]});
        assert_eq!(
            parse(Provider::Ollama, &value).as_deref(),
            Ok("Use lsof -i.")
        );
    }

    #[test]
    fn rejects_empty_response() {
        assert_eq!(
            parse(Provider::Compat(Compat::Groq), &json!({})).unwrap_err(),
            "groq returned no text"
        );
    }
}
