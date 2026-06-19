use crate::models::api::{ClientAiMode, ClientLlmProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiBillingMode {
    Managed,
    Byok,
}

impl AiBillingMode {
    pub fn from_profile(profile: Option<&ClientLlmProfile>) -> Self {
        match profile.map(|profile| profile.mode) {
            Some(ClientAiMode::Byok) => Self::Byok,
            _ => Self::Managed,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Managed => "managed",
            Self::Byok => "byok",
        }
    }
}

/// Placeholder for subscription checks and usage metering.
#[derive(Debug, Clone)]
pub struct AiEntitlements;

impl AiEntitlements {
    pub fn new() -> Self {
        Self
    }

    pub fn record_workflow_start(&self, mode: AiBillingMode, workflow: &'static str) {
        tracing::info!(
            ai_mode = mode.as_str(),
            workflow,
            "AI workflow entitlement check passed"
        );
    }
}

impl Default for AiEntitlements {
    fn default() -> Self {
        Self::new()
    }
}
