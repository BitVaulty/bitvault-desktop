use eframe::egui;
use crate::state::{AppState, Navigation, View};
use crate::ui::dashboard;
use crate::ui::vault_creation::{VaultCreationState, render as render_vault_creation};
use crate::ui::vault_selection::{VaultSelectionState, render as render_vault_selection};
use crate::ui::send_transaction::{SendTransactionState, render as render_send_transaction};
use crate::ui::receive::render as render_receive;
use crate::ui::settings::render as render_settings;
use crate::ui::transaction_detail::render as render_transaction_detail;
use crate::ui::recovery::{render_recovery, render_utxo_refresh};
use crate::ui::subscription::render as render_subscription;
use crate::ui::pin::{PinEntryState, PinSetupState, render_pin_entry, render_pin_setup};
use crate::ui::help::{render_help_and_support, HelpAndSupportState};
use crate::ui::address_book::{render_address_book, AddressBookState};
use crate::ui::advanced_settings::{render_advanced_settings, AdvancedSettingsState};

pub struct BitVaultApp {
    app_state: AppState,
    navigation: Navigation,
    vault_selection_state: VaultSelectionState,
    vault_creation_state: VaultCreationState,
    send_transaction_state: SendTransactionState,
    pin_entry_state: PinEntryState,
    pin_setup_state: PinSetupState,
    help_and_support_state: HelpAndSupportState,
    address_book_state: AddressBookState,
    advanced_settings_state: AdvancedSettingsState,
    is_authenticated: bool, // Whether user has entered PIN
}

impl BitVaultApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Try to load network from settings, default to Testnet
        let initial_network = {
            let settings_manager = crate::settings::SettingsManager::new()
                .unwrap_or_else(|_| {
                    eprintln!("Failed to create settings manager, using defaults");
                    crate::settings::SettingsManager::new().unwrap()
                });
            
            if let Ok(Some(network_str)) = settings_manager.get_network() {
                match network_str.as_str() {
                    "mainnet" => bdk::bitcoin::Network::Bitcoin,
                    "testnet" => bdk::bitcoin::Network::Testnet,
                    "signet" => bdk::bitcoin::Network::Signet,
                    "regtest" => bdk::bitcoin::Network::Regtest,
                    _ => bdk::bitcoin::Network::Testnet,
                }
            } else {
                bdk::bitcoin::Network::Testnet
            }
        };
        
        let app_state = AppState::new(initial_network)
            .unwrap_or_else(|e| {
                eprintln!("Failed to initialize app state: {}", e);
                // Fallback to default if settings fail
                AppState::new(bdk::bitcoin::Network::Testnet).unwrap()
            });
        
        // Check if PIN is required
        let pin_service = bitvault_common::PinService::new();
        let requires_pin = pin_service.has_pin();
        let is_authenticated = !requires_pin; // If no PIN is set, user is already "authenticated"
        
        Self {
            app_state,
            navigation: Navigation::new(),
            vault_selection_state: VaultSelectionState::default(),
            vault_creation_state: VaultCreationState::default(),
            send_transaction_state: SendTransactionState::default(),
            pin_entry_state: PinEntryState::new(),
            pin_setup_state: PinSetupState::new(),
            help_and_support_state: HelpAndSupportState::new(),
            address_book_state: AddressBookState::default(),
            advanced_settings_state: AdvancedSettingsState::default(),
            is_authenticated,
        }
    }

    /// Set the runtime for async operations
    pub fn set_runtime(&mut self, runtime: tokio::runtime::Runtime) {
        self.app_state.set_runtime(runtime);
    }
}

impl eframe::App for BitVaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process async commands and results
        self.app_state.process_async(Some(ctx));
        
        // Top bar (if needed)
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("BitVault");
                
                // Show current vault info if loaded
                if let Some(metadata) = self.app_state.get_current_vault_metadata() {
                    ui.separator();
                    ui.label(format!("{} ({})", metadata.name, metadata.network));
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("← Back").clicked() {
                        self.navigation.go_back();
                    }
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Check if PIN authentication is required
            if !self.is_authenticated {
                let mut callback = None;
                let pin_validated = render_pin_entry(ui, &mut self.pin_entry_state, &mut callback, ctx);
                
                if pin_validated {
                    self.is_authenticated = true;
                    // Navigate to appropriate screen after authentication
                    if self.app_state.is_vault_loaded() {
                        self.navigation.navigate_to(View::Dashboard { tab: 0 });
                    } else {
                        self.navigation.navigate_to(View::VaultSelection);
                    }
                }
                return; // Don't show other content until authenticated
            }
            
            let current_view = self.navigation.current_view.clone();
            match current_view {
                View::VaultSelection => {
                    render_vault_selection(ui, &mut self.app_state, &mut self.navigation, &mut self.vault_selection_state, ctx);
                }
                View::Dashboard { tab } => {
                    // If no vault is loaded, redirect to vault selection
                    if !self.app_state.is_vault_loaded() {
                        self.navigation.navigate_to(View::VaultSelection);
                    } else {
                        dashboard::render_dashboard(ui, &mut self.app_state, &mut self.navigation, tab);
                    }
                }
                View::VaultCreation => {
                    render_vault_creation(ui, &mut self.app_state, &mut self.navigation, &mut self.vault_creation_state);
                }
                View::SendTransaction => {
                    render_send_transaction(ui, &mut self.app_state, &mut self.navigation, &mut self.send_transaction_state);
                }
                View::Receive => {
                    render_receive(ui, &mut self.app_state, &mut self.navigation, ctx);
                }
                View::TransactionDetail { txid } => {
                    render_transaction_detail(ui, &mut self.app_state, &mut self.navigation, &txid);
                }
                View::Settings => {
                    render_settings(ui, &mut self.app_state, &mut self.navigation);
                }
                View::Recovery => {
                    render_recovery(ui, &mut self.app_state, &mut self.navigation);
                }
                View::UtxoRefresh => {
                    render_utxo_refresh(ui, &mut self.app_state, &mut self.navigation);
                }
                View::Subscription => {
                    render_subscription(ui, &mut self.app_state, &mut self.navigation);
                }
                View::PinEntry => {
                    let mut callback = None;
                    let pin_validated = render_pin_entry(ui, &mut self.pin_entry_state, &mut callback, ctx);
                    
                    if pin_validated {
                        // PIN validated - navigate to appropriate screen
                        if self.app_state.is_vault_loaded() {
                            self.navigation.navigate_to(View::Dashboard { tab: 0 });
                        } else {
                            self.navigation.navigate_to(View::VaultSelection);
                        }
                    }
                }
                View::PinSetup => {
                    let mut callback = None;
                    let pin_set = render_pin_setup(ui, &mut self.pin_setup_state, &mut callback);
                    
                    if pin_set {
                        // PIN set - continue with vault creation
                        // Navigation will be handled by vault creation flow
                    }
                }
                View::HelpAndSupport => {
                    render_help_and_support(ui, &mut self.app_state, &mut self.navigation, &mut self.help_and_support_state);
                }
                View::AddressBook => {
                    render_address_book(ui, &mut self.app_state, &mut self.navigation, &mut self.address_book_state, ctx);
                }
                View::AdvancedSettings => {
                    render_advanced_settings(ui, &mut self.app_state, &mut self.navigation, &mut self.advanced_settings_state);
                }
            }
        });
    }
}
