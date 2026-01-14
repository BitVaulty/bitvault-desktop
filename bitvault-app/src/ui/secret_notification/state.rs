//! State for secret notification setup

/// State for secret notification setup
#[derive(Default)]
pub struct SecretNotificationState {
    pub is_loading: bool,
    pub error: Option<String>,
    pub registration_link: Option<String>,
}

impl SecretNotificationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
