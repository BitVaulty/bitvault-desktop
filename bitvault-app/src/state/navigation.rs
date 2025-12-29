//! Navigation/routing
//!
//! Manages navigation state and current view

/// Current view/screen in the application
#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub enum View {
    /// Vault selection screen (shown on startup if no vault loaded)
    #[default]
    VaultSelection,
    /// Dashboard with tabs
    Dashboard {
        /// Current tab (0 = vault detail, 1 = transaction history, 2 = settings)
        tab: usize,
    },
    /// Vault creation flow
    VaultCreation,
    /// Send transaction flow
    SendTransaction,
    /// Receive address view
    Receive,
    /// Transaction details
    TransactionDetail {
        txid: String,
    },
    /// Settings (full screen)
    Settings,
    /// Recovery transaction flow
    Recovery,
    /// UTXO refresh flow
    UtxoRefresh,
    /// Subscription management
    Subscription,
    /// PIN entry (authentication)
    PinEntry,
    /// PIN setup (during vault creation)
    PinSetup,
    /// Help and Support
    HelpAndSupport,
    /// Address Book
    AddressBook,
    /// Advanced Settings
    AdvancedSettings,
}


/// Navigation state
pub struct Navigation {
    /// Current view
    pub current_view: View,
    /// View history (for back navigation)
    pub history: Vec<View>,
}

impl Navigation {
    /// Create new navigation state
    pub fn new() -> Self {
        Self {
            current_view: View::default(),
            history: Vec::new(),
        }
    }

    /// Navigate to a new view
    pub fn navigate_to(&mut self, view: View) {
        self.history.push(self.current_view.clone());
        self.current_view = view;
    }

    /// Navigate back
    pub fn go_back(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.current_view = previous;
            true
        } else {
            false
        }
    }

    /// Switch dashboard tab
    pub fn set_dashboard_tab(&mut self, tab: usize) {
        if tab < 3 {
            self.current_view = View::Dashboard { tab };
        }
    }
}
