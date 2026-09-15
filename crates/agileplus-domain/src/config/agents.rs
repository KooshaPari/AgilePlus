use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    #[serde(default = "default_agent")]
    pub default_agent: String,
    #[serde(default = "default_max_subagents")]
    pub max_subagents: u32,
    #[serde(default = "default_max_review_cycles")]
    pub max_review_cycles: u32,
    #[serde(default = "default_review_poll_interval")]
    pub review_poll_interval_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_agent: default_agent(),
            max_subagents: default_max_subagents(),
            max_review_cycles: default_max_review_cycles(),
            review_poll_interval_secs: default_review_poll_interval(),
        }
    }
}

fn default_agent() -> String {
    "claude-code".to_string()
}

fn default_max_subagents() -> u32 {
    3
}

fn default_max_review_cycles() -> u32 {
    5
}

fn default_review_poll_interval() -> u64 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_config() {
        let c = AgentConfig::default();
        assert_eq!(c.default_agent, "claude-code");
        assert_eq!(c.max_subagents, 3);
        assert_eq!(c.max_review_cycles, 5);
        assert_eq!(c.review_poll_interval_secs, 30);
    }

    #[test]
    fn serde_roundtrip() {
        let c = AgentConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.default_agent, c.default_agent);
        assert_eq!(back.max_subagents, c.max_subagents);
    }

    #[test]
    fn custom_values() {
        let c = AgentConfig {
            default_agent: "codex".into(),
            max_subagents: 10,
            max_review_cycles: 2,
            review_poll_interval_secs: 5,
        };
        assert_eq!(c.default_agent, "codex");
        assert_eq!(c.max_subagents, 10);
    }
}
