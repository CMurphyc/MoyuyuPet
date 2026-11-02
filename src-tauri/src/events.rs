#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl TokenUsage {
    pub fn total(self) -> u64 {
        self.input.saturating_add(self.output)
    }
}

#[derive(Clone, serde::Serialize)]
pub struct ActivityPayload {
    pub state: &'static str,
    pub intensity: f32,
    pub token_rate: f64,
    pub source: &'static str,
}

#[derive(Clone, serde::Serialize)]
pub struct BubblePayload {
    pub kind: &'static str,
    pub tool: Option<&'static str>,
    pub tokens: Option<u64>,
}

#[derive(Clone, serde::Serialize)]
pub struct AdapterStatusPayload {
    pub source: &'static str,
    pub status: &'static str,
    pub message: String,
}
