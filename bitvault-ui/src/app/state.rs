use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct GreetArgs<'a> {
    pub name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum WalletState {
    #[default]
    New,
    Creating,
    Restoring,
    Unlocked,
    Locked,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum View {
    #[default]
    Home,
    Disclaimer,
    PinChoice,
    Seed,
    SeedVerify,
    Wallet,
    LockScreen,
    SplashScreen,
    OnboardingOne,
    OnboardingTwo,
    OnboardingThree,
}

// Define a struct to hold the global state
#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub user_pin: Option<String>,
    pub wallet_state: WalletState,
    pub current_view: View,
    pub pin_input: String,
    pub pin_confirm: String,
    pub seed_phrase: Option<String>,
    pub seed_words: Vec<String>,
    pub verification_input: String,
    pub copied_feedback: Option<f32>, // Timer for showing copy feedback (in seconds)
    pub encrypted_wallet_data: Option<String>, // Encrypted wallet data stored on disk
    pub lock_error: Option<String>,   // Error message when unlocking fails
    pub splash_timer: Option<f32>,    // Timer for splash screen (in seconds)
    pub testing_mode: bool,           // Flag for testing mode to bypass lock screen
    pub onboarding_completed: bool,   // Flag to track if onboarding has been completed
}

// Create a type alias for a thread-safe, shared reference to the state
pub type SharedAppState = Arc<RwLock<AppState>>;
