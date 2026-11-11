//! Server-sent events reader shared by every streaming HTTP provider.

use super::{Provider, non_empty};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};

pub(super) type Response = ureq::http::Response<ureq::Body>;

const DATA_PREFIX: &str = "data:";
const DONE_MARKER: &str = "[DONE]";

/// Accumulates streamed text while forwarding each delta to the caller.
pub(super) struct Deltas<'a> {
    text: String,
    on_delta: &'a mut dyn FnMut(&str),
}

impl<'a> Deltas<'a> {
    pub(super) fn new(on_delta: &'a mut dyn FnMut(&str)) -> Self {
        Self {
            text: String::new(),
            on_delta,
        }
    }

    pub(super) fn push(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        self.text.push_str(delta);
        (self.on_delta)(delta);
    }

    pub(super) fn finish(self, provider: Provider) -> Result<String, String> {
        non_empty(provider, self.text)
    }
}

/// Servers that ignore `"stream": true` answer with plain JSON instead.
pub(super) fn is_event_stream(response: &Response) -> bool {
    response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("event-stream"))
}

pub(super) fn body(response: &mut Response) -> BufReader<impl Read + '_> {
    BufReader::new(response.body_mut().as_reader())
}

/// Calls `on_event` with each JSON `data:` payload until `[DONE]` or EOF.
pub(super) fn read_events(
    provider: Provider,
    reader: impl BufRead,
    on_event: &mut dyn FnMut(&Value) -> Result<(), String>,
) -> Result<(), String> {
    let mut data = String::new();
    for line in reader.lines() {
        let line = line.map_err(|error| format!("{provider} stream failed: {error}"))?;
        if !line.is_empty() {
            append_data(&mut data, &line);
            continue;
        }
        if dispatch(&data, on_event)? {
            return Ok(());
        }
        data.clear();
    }
    dispatch(&data, on_event).map(drop)
}

/// Formats an in-stream `error` payload from any provider.
pub(super) fn stream_error(provider: Provider, value: &Value) -> String {
    let error = &value["error"];
    let message = error["message"]
        .as_str()
        .or_else(|| error.as_str())
        .map_or_else(|| error.to_string(), str::to_owned);
    format!("{provider} stream error: {message}")
}

fn append_data(data: &mut String, line: &str) {
    let Some(payload) = line.strip_prefix(DATA_PREFIX) else {
        return;
    };
    if !data.is_empty() {
        data.push('\n');
    }
    data.push_str(payload.trim());
}

/// Returns true once the stream signalled `[DONE]`.
fn dispatch(
    data: &str,
    on_event: &mut dyn FnMut(&Value) -> Result<(), String>,
) -> Result<bool, String> {
    let trimmed = data.trim();
    if trimmed == DONE_MARKER {
        return Ok(true);
    }
    match serde_json::from_str::<Value>(trimmed) {
        Ok(value) => on_event(&value).map(|()| false),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::{Deltas, Provider, read_events, stream_error};
    use serde_json::{Value, json};

    fn collect(input: &str) -> Result<Vec<Value>, String> {
        let mut events = Vec::new();
        read_events(Provider::OpenAi, input.as_bytes(), &mut |value| {
            events.push(value.clone());
            Ok(())
        })?;
        Ok(events)
    }

    #[test]
    fn splits_events_on_blank_lines_and_skips_noise() {
        let input = ": keep-alive\nevent: delta\ndata: {\"n\":1}\n\ndata: {\"n\":2}\n\n";
        assert_eq!(collect(input), Ok(vec![json!({"n": 1}), json!({"n": 2})]));
    }

    #[test]
    fn joins_multi_line_data_and_reads_unterminated_tail() {
        let input = "data: {\"n\":\ndata: 3}";
        assert_eq!(collect(input), Ok(vec![json!({"n": 3})]));
    }

    #[test]
    fn stops_at_done_marker() {
        let input = "data: {\"n\":1}\n\ndata: [DONE]\n\ndata: {\"n\":2}\n\n";
        assert_eq!(collect(input), Ok(vec![json!({"n": 1})]));
    }

    #[test]
    fn propagates_handler_errors() {
        let result = read_events(Provider::Claude, "data: {}\n\n".as_bytes(), &mut |_| {
            Err("boom".to_owned())
        });
        assert_eq!(result, Err("boom".to_owned()));
    }

    #[test]
    fn formats_error_payloads() {
        let value = json!({"error": {"message": "Overloaded"}});
        assert_eq!(
            stream_error(Provider::Claude, &value),
            "claude stream error: Overloaded"
        );
    }

    #[test]
    fn deltas_forward_and_accumulate() {
        let mut seen = Vec::new();
        let mut on_delta = |delta: &str| seen.push(delta.to_owned());
        let mut deltas = Deltas::new(&mut on_delta);
        deltas.push("a");
        deltas.push("");
        deltas.push("b");
        assert_eq!(deltas.finish(Provider::OpenAi).as_deref(), Ok("ab"));
        assert_eq!(seen, ["a", "b"]);
    }
}
