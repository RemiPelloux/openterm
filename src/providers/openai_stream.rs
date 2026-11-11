use super::ask_prompt::INSTRUCTIONS;
use super::{AskRequest, Provider, http_agent, http_error, model, non_empty, read_json};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};

pub fn ask_stream(request: &AskRequest, on_delta: &mut dyn FnMut(&str)) -> Result<String, String> {
    let mut response = start_stream(request)?;
    if is_event_stream(&response) {
        return read_sse(&mut response, on_delta);
    }
    finish_json(&mut response, on_delta)
}

fn start_stream(request: &AskRequest) -> Result<ureq::http::Response<ureq::Body>, String> {
    let api_key = request.endpoint.api_key(Provider::OpenAi)?;
    let body = json!({
        "model": model(request)?,
        "instructions": INSTRUCTIONS,
        "input": request.prompt,
        "prompt_cache_key": super::openai::CACHE_KEY,
        "stream": true,
    });
    http_agent()
        .post(request.endpoint.url("v1/responses"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "text/event-stream")
        .send_json(body)
        .map_err(|error| http_error(Provider::OpenAi, &error))
}

fn finish_json(
    response: &mut ureq::http::Response<ureq::Body>,
    on_delta: &mut dyn FnMut(&str),
) -> Result<String, String> {
    let value = read_json(Provider::OpenAi, response)?;
    super::openai::report_usage(&value);
    let text = super::openai::parse_text(&value)?;
    if !text.is_empty() {
        on_delta(&text);
    }
    Ok(text)
}

fn is_event_stream(response: &ureq::http::Response<ureq::Body>) -> bool {
    response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("event-stream"))
}

fn read_sse(
    response: &mut ureq::http::Response<ureq::Body>,
    on_delta: &mut dyn FnMut(&str),
) -> Result<String, String> {
    let reader = BufReader::new(response.body_mut().as_reader());
    let mut raw = String::new();
    let mut data = String::new();
    for line in reader.lines() {
        let line = line.map_err(|error| format!("openai stream failed: {error}"))?;
        if line.is_empty() {
            take_event(&data, &mut raw, on_delta);
            data.clear();
            continue;
        }
        append_data(&mut data, &line);
    }
    take_event(&data, &mut raw, on_delta);
    let _ = std::io::stdout().flush();
    non_empty(Provider::OpenAi, raw)
}

fn append_data(data: &mut String, line: &str) {
    let Some(payload) = line.strip_prefix("data:") else {
        return;
    };
    if !data.is_empty() {
        data.push('\n');
    }
    data.push_str(payload.trim());
}

fn take_event(data: &str, raw: &mut String, on_delta: &mut dyn FnMut(&str)) {
    let Some(value) = parse_data(data) else {
        return;
    };
    if let Some(delta) = delta_text(&value) {
        raw.push_str(delta);
        on_delta(delta);
    }
    if is_completed(&value) {
        super::openai::report_usage(value.get("response").unwrap_or(&value));
    }
}

fn parse_data(data: &str) -> Option<Value> {
    let trimmed = data.trim();
    if trimmed.is_empty() || trimmed == "[DONE]" {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

fn delta_text(value: &Value) -> Option<&str> {
    let kind = value["type"].as_str().unwrap_or("");
    if kind.contains("output_text.delta") {
        return value["delta"]
            .as_str()
            .or_else(|| value["delta"]["text"].as_str());
    }
    value["choices"][0]["delta"]["content"].as_str()
}

fn is_completed(value: &Value) -> bool {
    value["type"].as_str() == Some("response.completed")
}

#[cfg(test)]
mod tests {
    use super::{delta_text, parse_data};
    use serde_json::json;

    #[test]
    fn parses_responses_delta_event() {
        let value =
            parse_data(r#"{"type":"response.output_text.delta","delta":"Yes."}"#).expect("json");
        assert_eq!(delta_text(&value), Some("Yes."));
    }

    #[test]
    fn ignores_done_marker() {
        assert!(parse_data("[DONE]").is_none());
    }

    #[test]
    fn parses_chat_completion_delta() {
        let value = json!({"choices":[{"delta":{"content":"Hi"}}]});
        assert_eq!(delta_text(&value), Some("Hi"));
    }
}
