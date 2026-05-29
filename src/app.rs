use crate::state::{AppState, Navigation, View};
use crate::ui::address_book::{render_address_book, AddressBookState};
use crate::ui::advanced_settings::{render_advanced_settings, AdvancedSettingsState};
use crate::ui::dashboard;
use crate::ui::help::{render_help_and_support, HelpAndSupportState};
use crate::ui::notification_center::{
    render as render_notification_center, NotificationCenterState,
};
use crate::ui::pin::{render_pin_entry, render_pin_setup, PinEntryState, PinSetupState};
use crate::ui::receive::render as render_receive;
use crate::ui::recovery::{render_recovery, render_utxo_refresh};
use crate::ui::secret_notification::{
    render as render_secret_notification, SecretNotificationState,
};
use crate::ui::send_transaction::{render as render_send_transaction, SendTransactionState};
use crate::ui::settings::render as render_settings;
use crate::ui::subscription::render as render_subscription;
use crate::ui::transaction_detail::render as render_transaction_detail;
use crate::ui::vault_creation::{render as render_vault_creation, VaultCreationState};
use crate::ui::vault_selection::{render as render_vault_selection, VaultSelectionState};
use eframe::egui;
use std::path::PathBuf;

/// Load the BitVault logo for display in the top bar
fn load_bitvault_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let mut possible_paths = vec![
        PathBuf::from("resources/bitvault_logo.png"),
        PathBuf::from("resources/bitvault_logo.svg"),
    ];

    // Add executable-relative paths
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // Try resources next to executable
            possible_paths.push(exe_dir.join("resources/bitvault_logo.png"));
            possible_paths.push(exe_dir.join("resources/bitvault_logo.svg"));

            // If we're in target/release, go up to find resources
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                let resources_logo = parent.join("resources/bitvault_logo.png");
                if resources_logo.exists() {
                    possible_paths.push(resources_logo.clone());
                    possible_paths.push(parent.join("resources/bitvault_logo.svg"));
                    break;
                }
                // Stop if we've gone too far up (reached root or workspace)
                if parent == current || !parent.exists() {
                    break;
                }
                current = parent;
            }
        }
    }

    for path in possible_paths.iter() {
        if path.exists() {
            if let Some(texture) = crate::utils::images::load_image_texture(ctx, path) {
                return Some(texture);
            }
        }
    }

    None
}

/// GitHub releases URL for desktop app updates
const DESKTOP_UPDATE_URL: &str = "https://github.com/BitVaulty/bitvault-desktop/releases";

pub struct BitVaultApp {
    app_state: AppState,
    navigation: Navigation,
    vault_selection_state: VaultSelectionState,
    vault_creation_state: VaultCreationState,
    send_transaction_state: SendTransactionState,
    pin_entry_state: PinEntryState,
    pin_setup_state: PinSetupState,
    help_and_support_state: HelpAndSupportState,
    secret_notification_state: SecretNotificationState,
    notification_center_state: NotificationCenterState,
    address_book_state: AddressBookState,
    advanced_settings_state: AdvancedSettingsState,
    is_authenticated: bool, // Whether user has entered PIN
    needs_pin_setup: bool,  // True if PIN needs to be set (doesn't exist yet)
    /// If true, app is outdated and user must update before using
    app_update_required: bool,
    cached_logo_texture: Option<egui::TextureHandle>, // Cache texture handle to keep it alive
    cached_logo_rect: Option<egui::Rect>, // Cache logo rect - recalculated on window resize
    last_screen_size: Option<egui::Vec2>, // Track screen size for resize detection
    last_pixels_per_point: Option<f32>,   // Track DPI for screen change detection
}

impl BitVaultApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        app_update_required: bool,
        remote_config: Option<bitvault_common::RemoteConfig>,
    ) -> Self {
        // Configure fonts - try to add system fonts with better Unicode support
        let mut fonts = egui::FontDefinitions::default();

        // Try to add system fonts that have better Unicode symbol support
        // This helps with displaying arrows, backspace, and other symbols
        #[cfg(target_os = "linux")]
        {
            // Try common Linux fonts with good Unicode coverage
            let font_paths = [
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ];

            for font_path in font_paths.iter() {
                if let Ok(font_data) = std::fs::read(font_path) {
                    // In egui 0.27, FontData is created directly from bytes
                    fonts.font_data.insert(
                        "noto_sans".to_string(),
                        egui::FontData::from_owned(font_data),
                    );
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        family.insert(0, "noto_sans".to_string());
                    }
                    break;
                }
            }
        }

        cc.egui_ctx.set_fonts(fonts);

        // Ensure panel backgrounds use system theme (not gray)
        // Don't override - let egui use default system theme
        let style = (*cc.egui_ctx.style()).clone();
        // Only adjust if we need to - for now use defaults
        cc.egui_ctx.set_style(style);

        // Install image loaders (including SVG support)
        egui_extras::install_image_loaders(&cc.egui_ctx);
        // Try to load network from settings, default to Testnet
        let initial_network = {
            // Try to create settings manager, but continue with defaults if it fails
            if let Ok(settings_manager) = crate::settings::SettingsManager::new() {
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
            } else {
                eprintln!("Warning: Failed to initialize settings manager. Using default network (Testnet).");
                eprintln!("Some settings features may be unavailable, but the app will continue to function.");
                bdk::bitcoin::Network::Testnet
            }
        };

        // Initialize app state with graceful error handling
        let mut app_state = AppState::new(initial_network).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to initialize app state with saved network: {}", e);
            eprintln!("Attempting fallback with default network (Testnet)...");

            // Try with default network as fallback
            AppState::new(bdk::bitcoin::Network::Testnet).unwrap_or_else(|e2| {
                eprintln!("CRITICAL: Failed to initialize app state even with default network: {}", e2);
                eprintln!("This usually indicates a system permissions issue (cannot access config directory).");
                eprintln!("Please check that the application has write permissions to: ~/.config/bitvault/");
                eprintln!("Attempting to continue with degraded functionality...");

                // Try fallback method
                AppState::new_without_settings(bdk::bitcoin::Network::Testnet).unwrap_or_else(|e3| {
                    eprintln!("FATAL: Cannot initialize application. Error: {}", e3);
                    eprintln!("Application will exit. Please check system permissions and try again.");
                    // Exit gracefully instead of panicking
                    std::process::exit(1);
                })
            })
        });
        #[cfg(feature = "native")]
        {
            app_state.remote_config = remote_config;
        }

        // Always require PIN setup/entry - show PIN screen on startup
        // If PIN exists, show entry screen; if not, show setup screen
        let pin_service = bitvault_common::PinService::new();
        let has_pin = pin_service.has_pin();
        eprintln!("[APP_INIT] has_pin: {}, will show PIN screen", has_pin);
        let is_authenticated = false; // Always start unauthenticated - must set/enter PIN
        let needs_pin_setup = !has_pin; // Show setup screen if no PIN exists

        Self {
            app_state,
            navigation: Navigation::new(),
            vault_selection_state: VaultSelectionState::default(),
            vault_creation_state: VaultCreationState::default(),
            send_transaction_state: SendTransactionState::default(),
            pin_entry_state: PinEntryState::new(),
            pin_setup_state: PinSetupState::new(),
            secret_notification_state: SecretNotificationState::new(),
            notification_center_state: NotificationCenterState::new(),
            needs_pin_setup,
            help_and_support_state: HelpAndSupportState::new(),
            address_book_state: AddressBookState::default(),
            advanced_settings_state: AdvancedSettingsState::default(),
            is_authenticated,
            app_update_required,
            cached_logo_texture: None,
            cached_logo_rect: None,
            last_screen_size: None,
            last_pixels_per_point: None,
        }
    }

    /// Set the runtime for async operations
    pub fn set_runtime(&mut self, runtime: tokio::runtime::Runtime) {
        self.app_state.set_runtime(runtime);
    }

    /// Update screenshot protection based on current sensitive screens
    fn update_screenshot_protection(&mut self) {
        use crate::services::screenshot_protection::ScreenshotProtection;
        use crate::state::View;
        use crate::ui::vault_creation::VaultCreationStep;

        // Determine if we're on a sensitive screen that should be protected
        let is_sensitive = match &self.navigation.current_view {
            View::PinEntry | View::PinSetup => true,
            View::VaultCreation => {
                // Check specific vault creation steps that show sensitive data
                matches!(
                    self.vault_creation_state.current_step,
                    VaultCreationStep::DisplaySeedPhrase
                        | VaultCreationStep::VerifySeedPhrase
                        | VaultCreationStep::SetPin
                        | VaultCreationStep::DisplayOwnKeys
                        | VaultCreationStep::DisplayExchangeData
                )
            }
            _ => false,
        };

        // Simple approach: enable/disable based on current screen
        // The API calls are lightweight on Windows, so calling each frame is acceptable
        // Future optimization: track state to avoid redundant calls
        if is_sensitive {
            // Try to enable protection
            if let Err(e) = ScreenshotProtection::enable() {
                // Only log if protection is available but failed
                // Silently ignore "not available" errors (e.g., on Linux)
                if ScreenshotProtection::is_available() {
                    log::warn!("Failed to enable screenshot protection: {}", e);
                }
            }
        } else {
            // Try to disable protection
            if let Err(e) = ScreenshotProtection::disable() {
                // Only log if protection was available
                if ScreenshotProtection::is_available() {
                    log::debug!("Failed to disable screenshot protection: {}", e);
                }
            }
        }
    }
}

impl eframe::App for BitVaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Manage screenshot protection based on current sensitive screens
        self.update_screenshot_protection();

        // Check for screen size or DPI changes FIRST (before any rendering)
        // This handles window moves between monitors with different scaling
        let screen_rect = ctx.screen_rect();
        let current_screen_size = screen_rect.size();
        let current_ppp = ctx.pixels_per_point();

        let size_changed = self
            .last_screen_size
            .is_some_and(|last_size| {
                (last_size.x - current_screen_size.x).abs() > 1.0
                    || (last_size.y - current_screen_size.y).abs() > 1.0
            });

        let ppp_changed = self
            .last_pixels_per_point
            .is_some_and(|last_ppp| (last_ppp - current_ppp).abs() > 0.01);

        if size_changed || ppp_changed {
            self.cached_logo_rect = None;
            // Request repaint to apply new scaling immediately
            ctx.request_repaint();
        }

        self.last_screen_size = Some(current_screen_size);
        self.last_pixels_per_point = Some(current_ppp);

        // Process async commands and results
        self.app_state.process_async(Some(ctx));

        // Modern top bar - set minimum height for larger logo
        egui::TopBottomPanel::top("top_bar")
            .min_height(48.0) // Minimum height to accommodate larger logo
            .show(ctx, |ui| {
                use crate::ui::components::{Colors, Spacing};

                // Get screen rect ONCE - this is TRULY stable and doesn't change with mouse
                let screen_rect = ctx.screen_rect();
                let _screen_center_x = screen_rect.center().x;

                // Get the clip rect for the top bar - this is the actual drawing area
                let clip_rect = ui.clip_rect();
                let _top_bar_y = clip_rect.min.y + clip_rect.height() / 2.0;

                // Background for top bar
                let available_rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    available_rect,
                    0.0,
                    if ctx.style().visuals.dark_mode {
                        Colors::GRAY_900
                    } else {
                        Colors::GRAY_50
                    },
                );

                // Draw BitVault logo - STABLE implementation
                // Cache texture handle (not just ID) to keep it alive
                if self.cached_logo_texture.is_none() {
                    if let Some(logo_texture) = load_bitvault_logo(ctx) {
                        self.cached_logo_texture = Some(logo_texture);
                    }
                }

                // Draw logo if we have cached texture
                if let Some(ref logo_texture) = self.cached_logo_texture {
                    // Calculate rect - recalculated when window resizes
                    let logo_rect = *self.cached_logo_rect.get_or_insert_with(|| {
                        // Get top bar height and calculate logo size to fit with margins
                        let top_bar_height = available_rect.height();
                        let margin = 8.0; // 8px margin on top and bottom
                        let max_logo_height = (top_bar_height - margin * 2.0).clamp(32.0, 40.0); // Larger logo: min 32px, max 40px

                        // Calculate width from aspect ratio
                        let texture_size = logo_texture.size_vec2();
                        let aspect_ratio = if texture_size.x > 0.0 && texture_size.y > 0.0 {
                            texture_size.y / texture_size.x
                        } else {
                            1.0
                        };
                        let logo_width = max_logo_height / aspect_ratio;

                        // Use screen center X and available rect center Y (within bounds)
                        let screen_center_x = screen_rect.center().x;
                        let logo_y = available_rect.center().y;

                        egui::Rect::from_center_size(
                            egui::pos2(screen_center_x, logo_y),
                            egui::Vec2::new(logo_width, max_logo_height),
                        )
                    });

                    // Draw using painter - outside layout system
                    ui.painter().image(
                        logo_texture.id(),
                        logo_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }

                ui.horizontal(|ui| {
                    // Left side: Back button first (more to the left, vertically centered)
                    if self.navigation.can_go_back() {
                        ui.add_space(Spacing::SM);
                        // Back button - narrower width for top bar (110px instead of 140px)
                        // Use allocate_exact_size to ensure fixed size and prevent hover size changes
                        // Add small top margin to push it down for proper vertical centering
                        let is_dark = ui.ctx().style().visuals.dark_mode;
                        let (bg_color, hover_color, text_color) = if is_dark {
                            (
                                egui::Color32::TRANSPARENT,
                                Colors::GRAY_800,
                                Colors::text_primary(ui.ctx()),
                            )
                        } else {
                            (
                                egui::Color32::TRANSPARENT,
                                Colors::GRAY_200,
                                Colors::GRAY_900,
                            )
                        };

                        // Pre-allocate exact size to prevent any size changes on hover
                        let desired_size = egui::Vec2::new(110.0, 38.0); // Narrower: 110px instead of 140px
                        let (mut rect, response) =
                            ui.allocate_exact_size(desired_size, egui::Sense::click());

                        // Move button down by adjusting rect position for proper vertical centering
                        rect = rect.translate(egui::vec2(0.0, 5.0));

                        // Draw button background and text
                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter();

                            // Draw background (transparent normally, hover color on hover)
                            let current_bg = if response.hovered() {
                                hover_color
                            } else {
                                bg_color
                            };
                            painter.rect_filled(rect, 4.0, current_bg);

                            // Draw text
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "← Back",
                                egui::FontId::proportional(14.0),
                                text_color,
                            );
                        }

                        // Handle click
                        if response.clicked() {
                            // Check if we're in a workflow that has step tracking
                            match self.navigation.current_view {
                                View::VaultCreation => {
                                    // Handle vault creation workflow step navigation
                                    if !self.vault_creation_state.go_to_previous_step() {
                                        // At first step, exit workflow
                                        self.navigation.go_back();
                                    }
                                }
                                View::Recovery => {
                                    // Handle recovery workflow step navigation
                                    use crate::ui::recovery::go_back_in_recovery_workflow;
                                    if !go_back_in_recovery_workflow() {
                                        // At first step, exit workflow
                                        self.navigation.go_back();
                                    }
                                }
                                View::UtxoRefresh => {
                                    // Handle UTXO refresh workflow step navigation
                                    use crate::ui::recovery::go_back_in_utxo_refresh_workflow;
                                    if !go_back_in_utxo_refresh_workflow() {
                                        // At first step, exit workflow
                                        self.navigation.go_back();
                                    }
                                }
                                _ => {
                                    // Not a workflow, use normal navigation
                                    self.navigation.go_back();
                                }
                            }
                        }
                    }

                    ui.add_space(Spacing::MD);

                    // Right side: Branding and vault info
                    // Show current vault info if loaded
                    if let Some(metadata) = self.app_state.get_current_vault_metadata() {
                        ui.add_space(Spacing::MD);
                        ui.separator();
                        ui.add_space(Spacing::MD);

                        // Vault name badge
                        use crate::ui::components::badge;
                        use crate::ui::components::BadgeStyle;
                        badge(ui, &metadata.name, BadgeStyle::Info);

                        ui.add_space(Spacing::SM);

                        // Network badge
                        let network_badge = match metadata.network.as_str() {
                            "mainnet" => BadgeStyle::Success,
                            "testnet" => BadgeStyle::Warning,
                            "signet" => BadgeStyle::Info,
                            _ => BadgeStyle::Neutral,
                        };
                        badge(ui, &metadata.network, network_badge);
                    }
                });

                ui.add_space(Spacing::SM);
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            // App update required - block all other content
            if self.app_update_required {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.heading("App Update Required");
                    ui.add_space(16.0);
                    ui.label(
                        "Please update to the latest version to continue using BitVault. \
                         Download the new version from the releases page.",
                    );
                    ui.add_space(32.0);
                    if ui.button("Update Now").clicked() {
                        ui.output_mut(|o| {
                            o.open_url = Some(egui::OpenUrl {
                                url: DESKTOP_UPDATE_URL.to_string(),
                                new_tab: true,
                            });
                        });
                    }
                });
                return;
            }

            // Check if PIN authentication/setup is required
            if !self.is_authenticated {
                if self.needs_pin_setup {
                    // Show PIN setup screen if no PIN exists
                    let mut callback = None;
                    let pin_set = render_pin_setup(ui, &mut self.pin_setup_state, &mut callback);

                    if pin_set {
                        eprintln!("[APP] PIN successfully set");
                        self.is_authenticated = true;
                        self.needs_pin_setup = false;
                        // Try to load active vault, or show vault selection
                        if self.app_state.try_load_active_vault().unwrap_or(false) {
                            self.navigation.set_view(View::Dashboard { tab: 0 });
                        } else {
                            self.navigation.set_view(View::VaultSelection);
                        }
                    }
                } else {
                    // Show PIN entry screen if PIN exists
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
                        self.is_authenticated = true;
                        // Set view without adding to history (PIN entry is not a workflow screen)
                        if self.app_state.is_vault_loaded() {
                            self.navigation.set_view(View::Dashboard { tab: 0 });
                        } else if self.app_state.try_load_active_vault().unwrap_or(false) {
                            // Try to load active vault for current network
                            self.navigation.set_view(View::Dashboard { tab: 0 });
                        } else {
                            self.navigation.set_view(View::VaultSelection);
                        }
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
                        // Verify vault metadata exists and database is valid
                        if let Some(metadata) = self.app_state.get_current_vault_metadata() {
                            // Check if database file exists
                            if !std::path::Path::new(&metadata.database_path).exists() {
                                // Database doesn't exist - unload vault and show selection
                                eprintln!("Vault database not found: {}", metadata.database_path);
                                self.app_state.unload_vault();
                                self.navigation.navigate_to(View::VaultSelection);
                            } else {
                                dashboard::render_dashboard(
                                    ui,
                                    &mut self.app_state,
                                    &mut self.navigation,
                                    tab,
                                );
                            }
                        } else {
                            // Can't find metadata - unload vault and show selection
                            eprintln!("Vault metadata not found - unloading vault");
                            self.app_state.unload_vault();
                            self.navigation.navigate_to(View::VaultSelection);
                        }
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
                View::SecretNotification => {
                    render_secret_notification(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.secret_notification_state,
                    );
                }
                View::NotificationCenter => {
                    // Fetch notifications on first load
                    if self.notification_center_state.last_fetch.is_none() {
                        if let Some(ref vault_service) = self.app_state.vault_service {
                            if let Some(runtime) = self.app_state.get_runtime() {
                                runtime.block_on(
                                    self.notification_center_state
                                        .fetch_notifications(vault_service),
                                );
                            }
                        }
                    }
                    render_notification_center(
                        ui,
                        &mut self.app_state,
                        &mut self.navigation,
                        &mut self.notification_center_state,
                    );
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
                        // PIN validated - set view without adding to history
                        if self.app_state.is_vault_loaded() {
                            self.navigation.set_view(View::Dashboard { tab: 0 });
                        } else {
                            self.navigation.set_view(View::VaultSelection);
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
