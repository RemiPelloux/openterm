pub struct LiveRender {
    color: bool,
    pending: String,
    fence: Option<String>,
    hiding: bool,
}

impl Default for LiveRender {
    fn default() -> Self {
        Self::new(crate::markdown::color_enabled())
    }
}

impl LiveRender {
    pub fn new(color: bool) -> Self {
        Self {
            color,
            pending: String::new(),
            fence: None,
            hiding: false,
        }
    }

    pub fn push(&mut self, delta: &str) -> String {
        self.pending.push_str(delta);
        let mut out = String::new();
        while let Some(idx) = self.pending.find('\n') {
            let line = self.pending[..idx].to_owned();
            self.pending.drain(..=idx);
            out.push_str(&self.feed_line(&line));
        }
        out
    }

    pub fn finish(&mut self) -> String {
        let mut out = String::new();
        if !self.pending.is_empty() {
            let line = std::mem::take(&mut self.pending);
            out.push_str(&self.feed_line(&line));
        }
        if self.fence.take().is_some() {
            out.push_str(&crate::markdown::close_fence(self.color));
        }
        out
    }
}

impl LiveRender {
    fn feed_line(&mut self, line: &str) -> String {
        if self.hiding {
            if crate::markdown::fence_lang(line).is_some() {
                self.hiding = false;
            }
            return String::new();
        }
        if let Some(lang) = crate::markdown::fence_lang(line) {
            return self.toggle_fence(lang);
        }
        if let Some(lang) = self.fence.as_deref() {
            return format!(
                "{}\n",
                crate::markdown::fence_body_line(lang, line, self.color)
            );
        }
        format!("{}\n", crate::markdown::render_line(line, self.color))
    }

    fn toggle_fence(&mut self, lang: &str) -> String {
        if self.fence.is_some() {
            self.fence = None;
            return crate::markdown::close_fence(self.color);
        }
        if lang == "opencode" {
            self.hiding = true;
            return String::new();
        }
        self.fence = Some(lang.to_owned());
        crate::markdown::open_fence(lang, self.color)
    }
}

#[cfg(test)]
mod tests {
    use super::LiveRender;

    #[test]
    fn streams_bold_lines_and_hides_opencode() {
        let mut live = LiveRender::new(false);
        let mut out = live.push("# Result\n1. **fast**\n```opencode\nEdit /tmp/x\n```\n");
        out.push_str(&live.finish());
        assert!(out.contains("Result"));
        assert!(out.contains("fast"));
        assert!(!out.contains("**"));
        assert!(!out.contains("Edit /tmp/x"));
        assert!(out.contains("•"));
    }
}
