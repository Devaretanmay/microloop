

use crate::config::{MicroloopConfig, MicroloopDefaults, Strictness, ToolConfig};
use crate::engine::RuleEngine;
use crate::history::HistoryTracker;

pub const ERROR_BUF_SIZE: usize = 1024;

pub struct MicroloopState {
    pub strictness: Strictness,
    pub max_repeats: usize,
    pub ignore_args: bool,
    pub history_window: Option<usize>,

    pub engine: RuleEngine,
    pub history: HistoryTracker,
    pub error_buffer: Vec<u8>,

    pub defaults: Option<MicroloopDefaults>,
    pub tools: Vec<ToolConfig>,
}

impl MicroloopState {
    pub fn new(yaml_str: &str) -> Result<Self, String> {
        let config = MicroloopConfig::from_yaml(yaml_str)?;

        let engine = RuleEngine::new(config.rules)?;
        let history = HistoryTracker::new();

        Ok(Self {
            strictness: config.strictness,
            max_repeats: config.max_repeats,
            ignore_args: config.ignore_args,
            history_window: config.history_window,
            engine,
            history,
            error_buffer: Vec::with_capacity(ERROR_BUF_SIZE),
            defaults: config.defaults,
            tools: config.tools,
        })
    }

    pub fn block_result(&self) -> u8 {
        match self.strictness {
            Strictness::Lenient => 1,
            Strictness::Balanced => 2,
            Strictness::Strict => 3,
        }
    }

    pub fn set_error(&mut self, msg: &str) {
        self.error_buffer.clear();

        let max_len = (ERROR_BUF_SIZE - 1).min(msg.len());
        self.error_buffer
            .extend_from_slice(&msg.as_bytes()[..max_len]);
        self.error_buffer.push(0);
    }
}
