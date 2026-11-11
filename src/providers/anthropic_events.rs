//! Anthropic Messages stream events, shared by the API and the Claude Code CLI
//! (whose `stream_event` lines wrap the same events).

use super::Provider;
use super::sse::{Deltas, stream_error};
use serde_json::Value;

/// Token counts gathered from `message_start` and `message_delta`.
#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct Usage {
    pub input: u64,
    pub output: u64,
    pub cached: u64,
}

impl Usage {
    pub(super) fn record(&self) {
        crate::hud::record_usage(self.input, self.output, self.cached);
    }
}

pub(super) struct MessageStream<'a> {
    provider: Provider,
    deltas: Deltas<'a>,
    usage: Usage,
}

impl<'a> MessageStream<'a> {
    pub(super) fn new(provider: Provider, on_delta: &'a mut dyn FnMut(&str)) -> Self {
        Self {
            provider,
            deltas: Deltas::new(on_delta),
            usage: Usage::default(),
        }
    }

    /// Applies one event; an `error` event ends the stream with its message.
    pub(super) fn take(&mut self, event: &Value) -> Result<(), String> {
        match event["type"].as_str() {
            Some("content_block_delta") if event["delta"]["type"] == "text_delta" => {
                self.deltas
                    .push(event["delta"]["text"].as_str().unwrap_or(""));
            }
            Some("message_start") => self.start(&event["message"]["usage"]),
            Some("message_delta") => {
                self.usage.output = count(&event["usage"], "output_tokens").max(self.usage.output);
            }
            Some("error") => return Err(stream_error(self.provider, event)),
            _ => {}
        }
        Ok(())
    }

    pub(super) const fn usage(&self) -> &Usage {
        &self.usage
    }

    pub(super) fn finish(self) -> Result<String, String> {
        self.deltas.finish(self.provider)
    }

    /// Input counts cache reads and writes too, matching the openai provider totals.
    fn start(&mut self, usage: &Value) {
        let cached = count(usage, "cache_read_input_tokens");
        self.usage = Usage {
            input: count(usage, "input_tokens")
                + cached
                + count(usage, "cache_creation_input_tokens"),
            output: count(usage, "output_tokens"),
            cached,
        };
    }
}

fn count(usage: &Value, field: &str) -> u64 {
    usage[field].as_u64().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{MessageStream, Provider, Usage};
    use serde_json::json;

    #[test]
    fn collects_text_deltas_and_usage() {
        let mut on_delta = |_: &str| {};
        let mut stream = MessageStream::new(Provider::Claude, &mut on_delta);
        let events = [
            json!({"type": "message_start", "message": {"usage": {
                "input_tokens": 5, "cache_read_input_tokens": 100,
                "cache_creation_input_tokens": 20, "output_tokens": 1}}}),
            json!({"type": "content_block_start", "index": 0}),
            json!({"type": "content_block_delta", "delta": {"type": "text_delta", "text": "du "}}),
            json!({"type": "content_block_delta", "delta": {"type": "thinking_delta", "thinking": "x"}}),
            json!({"type": "content_block_delta", "delta": {"type": "text_delta", "text": "-sh"}}),
            json!({"type": "message_delta", "usage": {"output_tokens": 7}}),
            json!({"type": "message_stop"}),
        ];
        for event in &events {
            stream.take(event).expect("event");
        }
        let expected = Usage {
            input: 125,
            output: 7,
            cached: 100,
        };
        assert_eq!(stream.usage(), &expected);
        assert_eq!(stream.finish().as_deref(), Ok("du -sh"));
    }

    #[test]
    fn surfaces_error_event() {
        let mut on_delta = |_: &str| {};
        let mut stream = MessageStream::new(Provider::Claude, &mut on_delta);
        let event = json!({"type": "error", "error": {"type": "overloaded_error", "message": "Overloaded"}});
        assert_eq!(
            stream.take(&event).unwrap_err(),
            "claude stream error: Overloaded"
        );
    }

    #[test]
    fn empty_stream_is_an_error() {
        let mut on_delta = |_: &str| {};
        let stream = MessageStream::new(Provider::Claude, &mut on_delta);
        assert_eq!(stream.finish().unwrap_err(), "claude returned no text");
    }
}
