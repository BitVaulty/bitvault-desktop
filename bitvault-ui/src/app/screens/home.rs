use eframe::egui::{self, Color32, RichText, Ui};

use crate::app::assets;
use crate::app::state::{SharedAppState, View, WalletState};
use crate::app::BitVaultApp;

// Main home screen
pub fn render(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.heading("Welcome to BitVault");
        ui.add_space(20.0);

        ui.label("Your secure Bitcoin wallet");
        ui.add_space(30.0);

        if ui.button("Create New Wallet").clicked() {
            if let Ok(mut state) = app.state.write() {
                state.wallet_state = WalletState::Creating;
                state.current_view = View::Disclaimer;
            }
        }

        ui.add_space(10.0);

        if ui.button("Restore Existing Wallet").clicked() {
            if let Ok(mut state) = app.state.write() {
                state.wallet_state = WalletState::Restoring;
                state.current_view = View::Disclaimer;
            }
        }

        let back_button_response = ui.add(egui::Button::new("Go Back"));
        if back_button_response.clicked() {
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::Home;
            }
        }
        crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
    });
}

// Disclaimer screen
pub fn render_disclaimer(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Important Disclaimer");
        ui.add_space(20.0);

        ui.label(RichText::new("Please read carefully before proceeding:").strong());
        ui.add_space(10.0);

        let disclaimer_text = "
        1. BitVault is a self-custody wallet. You are solely responsible for your funds.

        2. Your recovery phrase (seed) is the ONLY way to recover your wallet if you lose access.

        3. Never share your recovery phrase or PIN with anyone.

        4. Always back up your recovery phrase in a secure location.

        5. If you lose your recovery phrase, you will permanently lose access to your funds.

        6. BitVault cannot recover your wallet or funds if you lose your recovery phrase.
        ";

        ui.label(disclaimer_text);
        ui.add_space(20.0);

        if ui.button("I Understand and Accept").clicked() {
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::PinChoice;
            }
        }

        let back_button_response = ui.add(egui::Button::new("Go Back"));
        if back_button_response.clicked() {
            if let Ok(mut state) = app.state.write() {
                state.wallet_state = WalletState::New;
                state.current_view = View::Home;
            }
        }
        crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
    });
}

// Splash screen
pub fn render_splash_screen(ui: &mut Ui, _state: &SharedAppState) {
    // Set the background to black
    let screen_rect = ui.max_rect();
    ui.painter().rect_filled(screen_rect, 0.0, Color32::BLACK);

    // Track how many times this method is called
    static RENDER_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let count = RENDER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    log::trace!("Render splash screen called {} times", count);

    // Center the logo
    ui.vertical_centered(|ui| {
        // Use a static texture handle to avoid reloading on every frame
        static TEXTURE_ID: std::sync::OnceLock<Option<egui::TextureHandle>> =
            std::sync::OnceLock::new();

        let texture_id = TEXTURE_ID.get_or_init(|| {
            log::debug!("Loading splash logo - this should only happen once");
            assets::get_image_texture(ui.ctx(), "splash_logo", "public/splash_logo.png")
        });

        match texture_id {
            Some(texture) => {
                // Get texture size and available space
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();

                // Calculate appropriate scale - use a smaller maximum to prevent oversizing
                // Use a target width of 50-60% of screen width, but never larger than original
                let target_width_ratio = 0.5;
                let desired_width = available_size.x * target_width_ratio;
                let scale = (desired_width / texture_size.x).min(1.0);

                let display_size = texture_size * scale;

                // Ensure vertical centering by adjusting spacing
                let vertical_center_offset = (available_size.y - display_size.y) / 2.0;
                ui.add_space(vertical_center_offset);

                ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                log::trace!("Image added to frame {}", count);
            }
            None => {
                ui.colored_label(Color32::RED, "Failed to load splash image");
                log::error!("No texture available for splash screen");
            }
        }
    });

    // Request a repaint to ensure the timer updates even without mouse movement
    ui.ctx().request_repaint();
}
