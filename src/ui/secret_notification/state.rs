//! State for secret notification setup

use std::time::Instant;

/// Setup phase (matches iOS SecretNotification flow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecretNotificationPhase {
    #[default]
    Intro,
    LinkReady,
    WaitingForConfirmation,
    ConnectionFailed,
    Success,
}

/// State for secret notification setup
pub struct SecretNotificationState {
    pub phase: SecretNotificationPhase,
    pub is_loading: bool,
    pub error: Option<String>,
    pub registration_link: Option<String>,
    pub poll_attempts: u32,
    pub max_poll_attempts: u32,
    pub last_poll_at: Option<Instant>,
    pub poll_interval_secs: u64,
    pub retry_cooldown_until: Option<Instant>,
    pub retry_cooldown_secs: u64,
    pub telegram_opened: bool,
}

impl Default for SecretNotificationState {
    fn default() -> Self {
        Self {
            phase: SecretNotificationPhase::Intro,
            is_loading: false,
            error: None,
            registration_link: None,
            poll_attempts: 0,
            max_poll_attempts: 20,
            last_poll_at: None,
            poll_interval_secs: 3,
            retry_cooldown_until: None,
            retry_cooldown_secs: 5,
            telegram_opened: false,
        }
    }
}

impl SecretNotificationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn retry_cooldown_remaining_secs(&self) -> u64 {
        self.retry_cooldown_until
            .and_then(|until| until.checked_duration_since(Instant::now()))
            .map(|d| d.as_secs().saturating_add(1))
            .unwrap_or(0)
    }

    pub fn should_poll(&self) -> bool {
        if self.phase != SecretNotificationPhase::WaitingForConfirmation {
            return false;
        }
        if self.poll_attempts >= self.max_poll_attempts {
            return false;
        }
        match self.last_poll_at {
            None => true,
            Some(last) => last.elapsed().as_secs() >= self.poll_interval_secs,
        }
    }

    pub fn begin_waiting(&mut self) {
        self.phase = SecretNotificationPhase::WaitingForConfirmation;
        self.is_loading = true;
        self.error = None;
        self.poll_attempts = 0;
        self.last_poll_at = None;
        self.telegram_opened = true;
    }

    pub fn mark_poll_sent(&mut self) {
        self.last_poll_at = Some(Instant::now());
        self.poll_attempts = self.poll_attempts.saturating_add(1);
    }

    pub fn mark_registered(&mut self) {
        self.phase = SecretNotificationPhase::Success;
        self.is_loading = false;
        self.error = None;
    }

    pub fn mark_connection_failed(&mut self, message: String) {
        self.phase = SecretNotificationPhase::ConnectionFailed;
        self.is_loading = false;
        self.error = Some(message);
        self.retry_cooldown_until =
            Some(Instant::now() + std::time::Duration::from_secs(self.retry_cooldown_secs));
    }

    pub fn begin_retry(&mut self) {
        if self.retry_cooldown_remaining_secs() > 0 {
            return;
        }
        self.begin_waiting();
    }
}
