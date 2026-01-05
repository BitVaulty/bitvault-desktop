use crate::state::{AppState, Navigation, View};
use crate::ui::address_book::{render_address_book, AddressBookState};
use crate::ui::advanced_settings::{render_advanced_settings, AdvancedSettingsState};
use crate::ui::dashboard;
use crate::ui::help::{render_help_and_support, HelpAndSupportState};
use crate::ui::pin::{render_pin_entry, render_pin_setup, PinEntryState, PinSetupState};
use crate::ui::receive::render as render_receive;
use crate::ui::recovery::{render_recovery, render_utxo_refresh};
use crate::ui::send_transaction::{render as render_send_transaction, SendTransactionState};
use crate::ui::settings::render as render_settings;
use crate::ui::subscription::render as render_subscription;
use crate::ui::transaction_detail::render as render_transaction_detail;
use crate::ui::vault_creation::{render as render_vault_creation, VaultCreationState};
use crate::ui::vault_selection::{render as render_vault_selection, VaultSelectionState};
use eframe::egui;

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
            let settings_manager = match crate::settings::SettingsManager::new() {
                Ok(sm) => sm,
                Err(e) => {
                    eprintln!("Failed to create settings manager: {}, using defaults", e);
                    // Try one more time, but if it fails again, we'll panic (this shouldn't happen)
                    crate::settings::SettingsManager::new().unwrap_or_else(|e2| {
                        eprintln!(
                            "Critical: Failed to create settings manager twice: {}, {}",
                            e, e2
                        );
                        panic!("Cannot initialize settings manager");
                    })
                }
            };

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

        let app_state = match AppState::new(initial_network) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Failed to initialize app state with saved network: {}", e);
                // Fallback to default if settings fail
                match AppState::new(bdk::bitcoin::Network::Testnet) {
                    Ok(state) => state,
                    Err(e2) => {
                        eprintln!("CRITICAL: Failed to initialize app state even with default network: {}", e2);
                        eprintln!("This indicates a serious system error. The application may not function correctly.");
                        eprintln!(
                            "Attempting to continue anyway - some features may be unavailable."
                        );
                        // This should never happen, but if it does, we'll panic with a clear message
                        // rather than silently failing or using unsafe unwrap
                        panic!("FATAL: AppState initialization failed completely. This indicates a critical system error that prevents the application from starting. Error: {}", e2);
                    }
                }
            }
        };

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
                let runtime = self.app_state.get_runtime();
                let pin_validated =
                    render_pin_entry(ui, &mut self.pin_entry_state, &mut callback, ctx, runtime);

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
                    render_vault_selection(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.vault_selection_state,
                        ctx,
                    );
                }
                View::Dashboard { tab } => {
                    // If no vault is loaded, redirect to vault selection
                    if !self.app_state.is_vault_loaded() {
                        self.navigation.navigate_to(View::VaultSelection);
                    } else {
                        dashboard::render_dashboard(
                            ui,
                            &mut self.app_state,
                            &mut self.navigation,
                            tab,
                        );
                    }
                }
                View::VaultCreation => {
                    render_vault_creation(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.vault_creation_state,
                    );
                }
                View::SendTransaction => {
                    // Check if there's pre-filled address data from navigation
                    if let Some(prefilled_address) = self.navigation.take_navigation_data() {
                        self.send_transaction_state.recipient_address = prefilled_address;
                    }
                    render_send_transaction(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.send_transaction_state,
                    );
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
                    let runtime = self.app_state.get_runtime();
                    let pin_validated = render_pin_entry(
                        ui,
                        &mut self.pin_entry_state,
                        &mut callback,
                        ctx,
                        runtime,
                    );

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
                    render_help_and_support(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.help_and_support_state,
                    );
                }
                View::AddressBook => {
                    render_address_book(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.address_book_state,
                        ctx,
                    );
                }
                View::AdvancedSettings => {
                    render_advanced_settings(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.advanced_settings_state,
                    );
                }
            }
        });
    }
}
