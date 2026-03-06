//! Vault creation step implementations

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{button, button_large, ButtonStyle, Colors, Spacing};
use crate::ui::pin::render_pin_setup;
use crate::ui::vault_creation::{
    DeviceRole, HardwareWalletType, VaultCreationState, VaultCreationStep,
};
use crate::utils::icons::{icon_image, Icon};
use base64::{engine::general_purpose, Engine};
use bip39::{Language, Mnemonic};
use bitvault_common::key_exchange;
use bitvault_common::utils::TimeDelay;
use eframe::egui;
// Note: secp256k1 types removed - not currently used in this module
use std::collections::HashSet;

/// Role selection step - first step in vault creation
pub fn render_role_selection(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
    navigation: &mut Navigation,
) {
    ui.vertical_centered(|ui| {
        ui.heading("Set Up Vault");
        ui.add_space(Spacing::MD);

        ui.label("How would you like to set up your vault?");
        ui.add_space(Spacing::LG);
    });

    let icon_color = ui.style().visuals.text_color();
    let icon_size = 20.0;
    let card_width = 280.0;
    let card_height = 120.0;
    let row_width = card_width * 2.0 + Spacing::MD;

    // Center the grid
    let available_width = ui.available_width();
    let left_margin = ((available_width - row_width) / 2.0).max(0.0);

    // Row 1: View-Only and Create New
    ui.horizontal(|ui| {
        ui.add_space(left_margin);

        // Option 1: View-Only Mode
        let card1_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Import, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("View-Only Mode");
                            });
                            ui.add_space(Spacing::SM);
                            ui.label("Monitor without signing.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Set Up View-Only")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card1_response = ui.interact(
            card1_rect.response.rect,
            ui.id().with("card1"),
            egui::Sense::click(),
        );

        // Auto-focus first card only when step first changes (initial entry to this step)
        // This ensures Tab starts from the first card, not from other UI elements like Back button
        // step_just_changed() updates previous_step immediately, so this only runs once per step change
        if state.step_just_changed(VaultCreationStep::RoleSelection) {
            // Request focus on first card to set initial tab order
            // This only happens once when entering the step, won't interfere with subsequent tab navigation
            ui.memory_mut(|mem| mem.request_focus(card1_response.id));
        }

        let card1_keyboard = card1_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Draw focus indicator if focused
        if card1_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card1_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card1_response.clicked() || card1_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::ViewOnly;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card1_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        ui.add_space(Spacing::MD);

        // Option 2: Create New Vault
        let card2_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Plus, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("Create New Vault");
                            });
                            ui.add_space(Spacing::SM);
                            ui.label("Start a new vault.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Create New Vault")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card2_response = ui.interact(
            card2_rect.response.rect,
            ui.id().with("card2"),
            egui::Sense::click(),
        );
        let card2_keyboard = card2_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Draw focus indicator if focused
        if card2_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card2_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card2_response.clicked() || card2_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::Main;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card2_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    ui.add_space(Spacing::MD);

    // Row 2: Join as Co-owner and Restore
    ui.horizontal(|ui| {
        ui.add_space(left_margin);

        // Option 3: Join as Co-owner
        let card3_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Users, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("Join as Co-owner");
                            });
                            ui.add_space(Spacing::SM);
                            ui.label("Pair with main device.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Join as Co-owner")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card3_response = ui.interact(
            card3_rect.response.rect,
            ui.id().with("card3"),
            egui::Sense::click(),
        );
        let card3_keyboard = card3_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Draw focus indicator if focused
        if card3_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card3_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card3_response.clicked() || card3_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::Coowner;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card3_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        ui.add_space(Spacing::MD);

        // Option 4: Restore from Backup
        let card4_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Back, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("Restore from Backup");
                            });
                            ui.add_space(Spacing::SM);
                            ui.colored_label(egui::Color32::YELLOW, "Requires paper backup.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Restore from Backup")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card4_response = ui.interact(
            card4_rect.response.rect,
            ui.id().with("card4"),
            egui::Sense::click(),
        );
        let card4_keyboard = card4_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Draw focus indicator if focused
        if card4_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card4_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card4_response.clicked() || card4_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::Restore;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card4_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    ui.add_space(Spacing::MD);

    // Row 3: Single Device Vaults (Seed+HW and HW+HW)
    ui.horizontal(|ui| {
        ui.add_space(left_margin);

        // Option 5: Single Device (Seed + Hardware Wallet)
        let card5_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Plus, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("Single Device");
                            });
                            ui.add_space(Spacing::XS);
                            ui.label(egui::RichText::new("Seed + Hardware Wallet").small());
                            ui.add_space(Spacing::SM);
                            ui.label("One device with seed + HW.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Set Up Single Device")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card5_response = ui.interact(
            card5_rect.response.rect,
            ui.id().with("card5"),
            egui::Sense::click(),
        );
        let card5_keyboard = card5_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        if card5_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card5_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card5_response.clicked() || card5_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::SingleDeviceSeedHW;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card5_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        ui.add_space(Spacing::MD);

        // Option 6: Single Device (Hardware Wallet + Hardware Wallet)
        let card6_rect = ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(card_width - 24.0, card_height - 24.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(img) =
                                    icon_image(ui.ctx(), Icon::Plus, icon_size, icon_color)
                                {
                                    ui.add(img);
                                }
                                ui.strong("Single Device");
                            });
                            ui.add_space(Spacing::XS);
                            ui.label(egui::RichText::new("HW + Hardware Wallet").small());
                            ui.add_space(Spacing::SM);
                            ui.label("One device with 2 HWs.");
                            ui.add_space(Spacing::MD);
                            ui.label(
                                egui::RichText::new("→ Set Up Single Device")
                                    .color(ui.style().visuals.hyperlink_color),
                            );
                        });
                    });
            },
        );
        let card6_response = ui.interact(
            card6_rect.response.rect,
            ui.id().with("card6"),
            egui::Sense::click(),
        );
        let card6_keyboard = card6_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        if card6_response.has_focus() {
            let painter = ui.painter();
            let outline_rect = card6_response.rect.expand(2.0);
            painter.rect_stroke(
                outline_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }

        if card6_response.clicked() || card6_keyboard {
            state.reset_for_new_flow();
            state.device_role = DeviceRole::SingleDeviceHWHW;
            state.advance_to_step(VaultCreationStep::NameVault);
        }
        if card6_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    ui.add_space(Spacing::XL);

    ui.vertical_centered(|ui| {
        if button(ui, "Cancel", ButtonStyle::Text).clicked() {
            navigation.go_back();
        }
    });
}

/// Name vault step
pub fn render_name_vault(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    let role_text = match state.device_role {
        DeviceRole::Main => "Main Device Setup",
        DeviceRole::Coowner => "Co-owner Setup",
        DeviceRole::ViewOnly => "View-Only Setup",
        DeviceRole::Restore => "Restore from Backup",
        DeviceRole::SingleDeviceSeedHW => "Single Device Setup (Seed + Hardware Wallet)",
        DeviceRole::SingleDeviceHWHW => "Single Device Setup (Hardware Wallet + Hardware Wallet)",
    };

    ui.vertical_centered(|ui| {
        ui.heading(role_text);
        ui.add_space(Spacing::LG);

        ui.label("Enter a name for this vault:");
        ui.add_space(Spacing::MD);

        let name_response = ui.add(
            egui::TextEdit::singleline(&mut state.vault_name)
                .hint_text("My Bitcoin Vault")
                .desired_width(300.0)
                .margin(egui::vec2(8.0, 6.0)),
        );

        // Auto-focus on step change
        if state.step_just_changed(VaultCreationStep::NameVault) {
            name_response.request_focus();
        }

        // Handle Enter key to submit
        let should_submit =
            name_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(Spacing::XL);

        let continue_response = button_large(ui, "Continue");
        let keyboard_clicked = continue_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
        if continue_response.clicked() || should_submit || keyboard_clicked {
            if state.vault_name.trim().is_empty() {
                state.error = Some("Please enter a vault name".to_string());
            } else {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }

        ui.add_space(Spacing::MD);
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Set time delay (main device only)
pub fn render_set_time_delay(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Set Time Delay");
        ui.add_space(Spacing::MD);

        ui.label("Choose how long before the fast-path becomes available.");
        ui.label("This is your security buffer if your device is compromised.");
        ui.add_space(Spacing::LG);

        // Days slider in a centered group
        ui.label("Days:");
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_width(400.0);
            ui.style_mut().spacing.slider_width = 380.0;
            ui.add(egui::Slider::new(&mut state.time_delay_days, 0..=365));
        });

        ui.add_space(Spacing::MD);

        // Hours slider in a centered group
        ui.label("Hours:");
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_width(400.0);
            ui.style_mut().spacing.slider_width = 380.0;
            ui.add(egui::Slider::new(&mut state.time_delay_hours, 0..=23));
        });

        ui.add_space(Spacing::MD);

        let total_hours = (state.time_delay_days * 24 + state.time_delay_hours) as f32;
        if total_hours < 24.0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ A time delay of at least 24 hours is recommended.",
            );
        }

        ui.add_space(Spacing::XL);

        let continue_response = button_large(ui, "Continue");
        let continue_keyboard = continue_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Allow Enter key to trigger Continue from anywhere on this step
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if continue_response.clicked() || continue_keyboard || enter_pressed {
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }

        ui.add_space(Spacing::MD);
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Generate mnemonic step
pub fn render_mnemonic_generation(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Create Seed Phrase");
        ui.add_space(Spacing::LG);

        ui.label("Your seed phrase is the key to your vault.");
        ui.label("It will be generated securely on this device.");
        ui.add_space(Spacing::MD);

        // Word count selection
        ui.label("Seed phrase length:");
        ui.add_space(Spacing::XS);
        ui.horizontal(|ui| {
            let word_count_12 = state.mnemonic_word_count == 12;
            let word_count_24 = state.mnemonic_word_count == 24;

            if ui.selectable_label(word_count_12, "12 words").clicked() {
                state.mnemonic_word_count = 12;
            }
            if ui.selectable_label(word_count_24, "24 words").clicked() {
                state.mnemonic_word_count = 24;
            }
        });

        ui.add_space(Spacing::MD);
        if state.mnemonic_word_count == 24 {
            ui.label(
                egui::RichText::new("24 words provide higher security (256 bits of entropy)")
                    .small(),
            );
        } else {
            ui.label(
                egui::RichText::new("12 words provide standard security (128 bits of entropy)")
                    .small(),
            );
        }

        ui.add_space(Spacing::XL);

        let generate_response = button_large(ui, "Generate Seed Phrase");
        let generate_keyboard = generate_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Allow Enter key to trigger Generate from anywhere on this step
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if generate_response.clicked() || generate_keyboard || enter_pressed {
            let word_count = state.mnemonic_word_count;
            let mnemonic = bitvault_common::generate_mnemonic(word_count);

            match mnemonic {
                Ok(mnemonic) => {
                    state.mnemonic = Some(mnemonic);
                    state.error = None;
                    state.advance_to_step(VaultCreationStep::DisplaySeedPhrase);
                }
                Err(e) => {
                    state.error = Some(format!("Failed to generate mnemonic: {}", e));
                }
            }
        }

        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(Spacing::MD);
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Display seed phrase
pub fn render_display_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Write Down Your Seed Phrase");
        ui.add_space(Spacing::MD);

        ui.colored_label(egui::Color32::RED, "⚠ IMPORTANT: Write this down on paper!");
        ui.label("Never share it. Never store it digitally. You need this to recover your vault.");
        ui.add_space(Spacing::LG);
    });

    // Center the seed phrase card
    if let Some(ref mnemonic) = state.mnemonic {
        // Use same method as verification to ensure consistency
        let mnemonic_str = mnemonic.to_string();
        let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
        let card_width = 380.0;
        let available_width = ui.available_width();
        let left_margin = ((available_width - card_width) / 2.0).max(0.0);

        ui.horizontal(|ui| {
            ui.add_space(left_margin);
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_width(card_width - 40.0); // Account for margins
                    egui::Grid::new("seed_words")
                        .num_columns(3)
                        .spacing([28.0, 14.0])
                        .show(ui, |ui| {
                            for (i, word) in words.iter().enumerate() {
                                ui.monospace(format!("{:2}. {}", i + 1, word));
                                if (i + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
    }

    ui.add_space(Spacing::XL);

    ui.vertical_centered(|ui| {
        let written_down_response = button_large(ui, "I've Written It Down");
        let written_down_keyboard = written_down_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        // Allow Enter key to trigger from anywhere on this step
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if written_down_response.clicked() || written_down_keyboard || enter_pressed {
            state.advance_to_step(VaultCreationStep::VerifySeedPhrase);
        }

        ui.add_space(Spacing::MD);
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Render hardware wallet type selection UI consistently across all flows
///
/// This helper function ensures consistent UX/UI for hardware wallet type selection
/// in co-owner key scanning, view-only setup, and restore flows.
fn render_hardware_wallet_type_selection(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
    grid_id: &str,
) {
    ui.add_space(Spacing::SM);
    ui.label("Select your hardware wallet type:");
    ui.add_space(Spacing::XS);

    // Create a grid of selectable hardware wallet type buttons
    let mut selected_type = state.selected_hw_type;
    egui::Grid::new(grid_id)
        .num_columns(2)
        .spacing([Spacing::SM, Spacing::SM])
        .show(ui, |ui| {
            for hw_type in HardwareWalletType::all_types() {
                let is_selected = selected_type == Some(hw_type);
                let button = egui::SelectableLabel::new(is_selected, hw_type.title());

                if ui.add(button).clicked() {
                    selected_type = Some(hw_type);
                }
            }
        });

    // Update state if selection changed
    if selected_type != state.selected_hw_type {
        state.selected_hw_type = selected_type;
        // Reset scanner when type changes
        if state.hw_batch_qr_scanner_state.success {
            state.hw_batch_qr_scanner_state.reset();
            if state.coowner_keys.is_some() {
                state.coowner_keys = None;
            }
            if !state.import_descriptors_qr.is_empty() {
                state.import_descriptors_qr.clear();
            }
        }
    }

    ui.add_space(Spacing::MD);

    // Show guidance based on selected type (consistent messaging)
    if let Some(hw_type) = state.selected_hw_type {
        if hw_type.uses_multi_part_ur() {
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 0),
                format!("⚠ {} uses multi-part UR codes. You'll need to scan multiple QR codes in sequence.", hw_type.title())
            );
        } else {
            ui.label(
                egui::RichText::new(format!("{} uses single-part UR codes.", hw_type.title()))
                    .weak(),
            );
        }
        ui.add_space(Spacing::SM);
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            "⚠ Please select your hardware wallet type above before scanning.",
        );
        ui.add_space(Spacing::SM);
    }
}

/// Helper function to format ordinal numbers (1st, 2nd, 3rd, etc.)
fn ordinal_number(n: usize) -> String {
    match n {
        1 => "1st".to_string(),
        2 => "2nd".to_string(),
        3 => "3rd".to_string(),
        n if n % 10 == 1 && n % 100 != 11 => format!("{}st", n),
        n if n % 10 == 2 && n % 100 != 12 => format!("{}nd", n),
        n if n % 10 == 3 && n % 100 != 13 => format!("{}rd", n),
        n => format!("{}th", n),
    }
}

/// Generate random distractor words from BIP39 wordlist
/// Excludes words that are in the user's mnemonic
///
/// Safety: BIP39 has 2048 words, so even with 12-word mnemonic, we have 2036+ words available.
/// This function will always succeed for reasonable count values (e.g., 24 distractors).
fn generate_distractor_words(exclude_words: &HashSet<&str>, count: usize) -> Vec<String> {
    let mut distractors = Vec::new();
    let max_attempts = 1000; // Safety limit to prevent infinite loops
    let mut attempts = 0;

    while distractors.len() < count && attempts < max_attempts {
        attempts += 1;
        // Generate random mnemonic
        let entropy: [u8; 16] = rand::random();
        if let Ok(mnemonic) = Mnemonic::from_entropy(&entropy) {
            // Extract words and filter
            for word in mnemonic.words() {
                if !exclude_words.contains(word) && !distractors.contains(&word.to_string()) {
                    distractors.push(word.to_string());
                    if distractors.len() >= count {
                        break;
                    }
                }
            }
        }
    }

    // If we couldn't get enough words (shouldn't happen in practice), return what we have
    if distractors.len() < count {
        eprintln!(
            "Warning: Only generated {} distractor words out of {} requested",
            distractors.len(),
            count
        );
    }

    distractors
}

/// Initialize seed phrase verification state
fn initialize_verification_state(state: &mut VaultCreationState) -> Result<(), String> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mnemonic = state
        .mnemonic
        .as_ref()
        .ok_or_else(|| "No mnemonic available".to_string())?;

    // Get words by splitting the mnemonic string to ensure correct order
    // mnemonic.words() should work too, but splitting is more explicit
    let mnemonic_str = mnemonic.to_string();
    let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
    // Support both 12 and 24 word mnemonics
    if words.len() != 12 && words.len() != 24 {
        return Err(format!("Expected 12 or 24 words, got {}", words.len()));
    }
    let exclude_words: HashSet<&str> = words.iter().copied().collect();

    // Create verification indices (0-11, shuffled)
    let mut indices: Vec<usize> = (0..words.len()).collect();
    indices.shuffle(&mut thread_rng());
    state.seed_verification_state.verification_indices = indices;

    // Generate word choices for each index
    for &word_idx in &state.seed_verification_state.verification_indices {
        // Ensure word_idx is valid
        if word_idx >= words.len() {
            return Err(format!(
                "Invalid word index: {} (mnemonic has {} words)",
                word_idx,
                words.len()
            ));
        }

        let correct_word = words[word_idx];
        let correct_word_str = correct_word.to_string();

        // Verify correct_word is actually from the mnemonic
        if !exclude_words.contains(correct_word) {
            return Err(format!(
                "Word at index {} is not in mnemonic: {}",
                word_idx, correct_word
            ));
        }

        // Generate 2 distractor words
        let distractors = generate_distractor_words(&exclude_words, 2);

        // Combine and shuffle - CRITICAL: correct word must be first in the vector before shuffling
        let mut choices = vec![correct_word_str.clone()];
        choices.extend(distractors);

        // Verify correct word is present before shuffling
        if !choices.iter().any(|w| w == &correct_word_str) {
            return Err(format!(
                "Correct word {} not in choices before shuffle for index {}",
                correct_word, word_idx
            ));
        }

        choices.shuffle(&mut thread_rng());

        // Verify correct word is still in choices after shuffling
        if !choices.iter().any(|w| w == &correct_word_str) {
            return Err(format!(
                "Correct word {} not found in choices after shuffle for index {}",
                correct_word, word_idx
            ));
        }

        state
            .seed_verification_state
            .word_choices
            .insert(word_idx, choices);
    }

    state.seed_verification_state.initialized = true;
    state.seed_verification_state.current_page = 0; // Start on first page
    Ok(())
}

/// Validate current page of seed phrase verification (6 words per page)
fn validate_current_page(state: &VaultCreationState) -> Result<(), String> {
    let page_size = 6;
    let current_page = state.seed_verification_state.current_page;

    // Determine total word count from mnemonic
    let total_words = if let Some(ref mnemonic) = state.mnemonic {
        let mnemonic_str = mnemonic.to_string();
        mnemonic_str.split_whitespace().count()
    } else {
        12 // Fallback
    };

    let start_idx = current_page * page_size;
    let end_idx = ((current_page + 1) * page_size).min(total_words);

    // Get the word indices for current page
    let page_indices: Vec<usize> =
        state.seed_verification_state.verification_indices[start_idx..end_idx].to_vec();

    // Check all 6 positions on current page are selected
    for &word_idx in &page_indices {
        if !state
            .seed_verification_state
            .selected_words
            .contains_key(&word_idx)
        {
            return Err(format!(
                "Please select all {} words on this page",
                page_size
            ));
        }
    }

    // Verify each selection on current page
    if let Some(ref mnemonic) = state.mnemonic {
        let mnemonic_str = mnemonic.to_string();
        let correct_words: Vec<&str> = mnemonic_str.split_whitespace().collect();

        for &word_idx in &page_indices {
            let selected_word = state
                .seed_verification_state
                .selected_words
                .get(&word_idx)
                .ok_or_else(|| "Word not selected".to_string())?;
            let correct_word = correct_words
                .get(word_idx)
                .ok_or_else(|| format!("Invalid word index: {}", word_idx))?;

            if *correct_word != selected_word {
                return Err("One or more words are incorrect. Please try again.".to_string());
            }
        }

        Ok(())
    } else {
        Err("No mnemonic available".to_string())
    }
}

/// Validate all words (final verification) - supports 12 or 24 words
fn validate_all_words(state: &VaultCreationState) -> Result<(), String> {
    // Determine expected word count from mnemonic
    let expected_count = if let Some(ref mnemonic) = state.mnemonic {
        let mnemonic_str = mnemonic.to_string();
        mnemonic_str.split_whitespace().count()
    } else {
        // Fallback to state setting
        state.mnemonic_word_count as usize
    };

    // Check all positions selected
    if state.seed_verification_state.selected_words.len() != expected_count {
        return Err(format!("Please select all {} words", expected_count));
    }

    // Verify each selection
    if let Some(ref mnemonic) = state.mnemonic {
        let mnemonic_str = mnemonic.to_string();
        let correct_words: Vec<&str> = mnemonic_str.split_whitespace().collect();

        for (index, selected_word) in &state.seed_verification_state.selected_words {
            let correct_word = correct_words
                .get(*index)
                .ok_or_else(|| format!("Invalid word index: {}", index))?;

            if *correct_word != selected_word {
                return Err("One or more words are incorrect. Please try again.".to_string());
            }
        }

        Ok(())
    } else {
        Err("No mnemonic available".to_string())
    }
}

/// Render a single word prompt with 3 toggle buttons
fn render_word_prompt(
    ui: &mut egui::Ui,
    _prompt_idx: usize,
    word_idx: usize,
    state: &mut VaultCreationState,
) {
    // Get choices and selected word before borrowing
    let choices = state
        .seed_verification_state
        .word_choices
        .get(&word_idx)
        .cloned();
    let selected_word = state
        .seed_verification_state
        .selected_words
        .get(&word_idx)
        .cloned();

    // Horizontal row with center vertical alignment
    // This ensures label and buttons are on same baseline
    ui.horizontal(|ui| {
        // Number label - show actual word position in seed phrase (1-based)
        // Fixed width area for label to ensure consistent spacing regardless of digit count
        // Use monospace and format with padding to ensure consistent width
        let label_text = format!("{:2}.", word_idx + 1); // Right-pad single digits: " 1." vs "12."
        let label_width = 35.0; // Fixed width for "12." (2 digits + period)

        // Allocate exact size and render label right-aligned
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(label_width, 36.0), // Fixed width, height matches button
            egui::Sense::hover(),
        );

        // Render label in the allocated rect, right-aligned
        ui.painter().text(
            egui::pos2(rect.max.x - 4.0, rect.center().y), // Right-aligned with small margin
            egui::Align2::RIGHT_CENTER,
            label_text,
            egui::TextStyle::Monospace.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        ui.add_space(Spacing::SM);

        // Three word buttons in a row
        if let Some(choices) = choices {
            for word in choices {
                let is_selected = selected_word.as_ref().map(|w| w == &word).unwrap_or(false);

                render_toggle_word_button(ui, word, is_selected, word_idx, state);
                ui.add_space(Spacing::SM); // Space between buttons
            }
        }
    });
}

/// Render a toggle button for a word choice
fn render_toggle_word_button(
    ui: &mut egui::Ui,
    word: String,
    is_selected: bool,
    word_idx: usize,
    state: &mut VaultCreationState,
) {
    // Button styling based on selection state
    let (bg_color, text_color, border_color) = if is_selected {
        // Selected: Primary color background, white text
        (Colors::PRIMARY, egui::Color32::WHITE, Colors::PRIMARY_DARK)
    } else {
        // Unselected: Transparent/gray background, normal text
        let is_dark = ui.style().visuals.dark_mode;
        let bg = if is_dark {
            Colors::GRAY_800
        } else {
            Colors::GRAY_100
        };
        (bg, Colors::text_primary(ui.ctx()), Colors::GRAY_400)
    };

    // Custom button using egui::Button with custom styling
    let button = egui::Button::new(
        egui::RichText::new(&word)
            .color(text_color)
            .monospace()
            .size(14.0),
    )
    .fill(bg_color)
    .min_size(egui::vec2(120.0, 36.0)); // Fixed width, consistent height

    let response = ui.add(button);

    // Draw border for selected state
    if is_selected {
        ui.painter().rect_stroke(
            response.rect,
            6.0, // Corner radius
            egui::Stroke::new(2.0, border_color),
        );
    }

    // Handle click: toggle selection
    if response.clicked() {
        if is_selected {
            // Deselect
            state
                .seed_verification_state
                .selected_words
                .remove(&word_idx);
        } else {
            // Select (this replaces any previous selection for this position)
            state
                .seed_verification_state
                .selected_words
                .insert(word_idx, word);
        }
    }

    // Hover effect
    if response.hovered() && !is_selected {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// Verify seed phrase
pub fn render_verify_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    // Initialize verification state if not already done
    if !state.seed_verification_state.initialized {
        if let Err(e) = initialize_verification_state(state) {
            state.error = Some(e);
        }
    }

    // Header section (centered)
    ui.vertical_centered(|ui| {
        ui.heading("Verify Seed Phrase");
        ui.add_space(Spacing::MD);
        ui.label("Please verify your seed phrase by selecting the correct word for each position.");
        ui.add_space(Spacing::LG);
    });

    // Main content card (centered, similar to seed phrase display)
    let card_width = 500.0; // Wider than seed phrase card (380.0) to fit 3 buttons
    let available_width = ui.available_width();
    let left_margin = ((available_width - card_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(left_margin);

        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(card_width - 40.0); // Account for margins

                // Display only current page (6 words at a time)
                // No scroll area needed - card fits exactly 6 prompts
                let page_size = 6;
                let current_page = state.seed_verification_state.current_page;

                // Determine total word count from mnemonic
                let total_words = if let Some(ref mnemonic) = state.mnemonic {
                    let mnemonic_str = mnemonic.to_string();
                    mnemonic_str.split_whitespace().count()
                } else {
                    12 // Fallback
                };

                let start_idx = current_page * page_size;
                let end_idx = ((current_page + 1) * page_size).min(total_words);

                // Clone indices to avoid borrow checker issues
                let indices = state.seed_verification_state.verification_indices.clone();
                let page_indices: Vec<usize> = indices[start_idx..end_idx].to_vec();

                // Use Grid to explicitly create rows (similar to seed phrase display)
                egui::Grid::new("word_verification_prompts")
                    .num_columns(1) // One column - each row is one prompt
                    .spacing([0.0, Spacing::SM]) // No horizontal spacing, vertical spacing between rows
                    .show(ui, |ui| {
                        // Render only current page (6 prompts)
                        for (local_idx, word_idx) in page_indices.iter().enumerate() {
                            let global_prompt_idx = start_idx + local_idx;
                            render_word_prompt(ui, global_prompt_idx, *word_idx, state);
                            ui.end_row(); // Explicitly end each row
                        }
                    });
            });
    });

    ui.add_space(Spacing::XL);

    // Footer section (centered)
    ui.vertical_centered(|ui| {
        // Error message area - always reserve space to prevent layout shift
        // Reserve space for error message (SM spacing + 2 lines of text + MD spacing)
        let error_area_height = 60.0; // Fixed height for error area

        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), error_area_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                if let Some(ref error) = state.error {
                    ui.add_space(Spacing::SM);
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(Spacing::MD);
                }
                // If no error, this area is empty but still takes up space
            },
        );

        // Button text changes based on page
        // Determine total pages based on word count
        let total_words = if let Some(ref mnemonic) = state.mnemonic {
            let mnemonic_str = mnemonic.to_string();
            mnemonic_str.split_whitespace().count()
        } else {
            12 // Fallback
        };
        let page_size = 6;
        let total_pages = total_words.div_ceil(page_size);
        let is_last_page = state.seed_verification_state.current_page == (total_pages - 1);
        let button_text = if is_last_page { "Continue" } else { "Next" };

        // Continue/Next button
        let continue_response = button_large(ui, button_text);
        let continue_keyboard = continue_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));

        if continue_response.clicked() || continue_keyboard {
            // Validate current page
            match validate_current_page(state) {
                Ok(()) => {
                    state.error = None;

                    // If not last page, advance to next page
                    if !is_last_page {
                        state.seed_verification_state.current_page += 1;
                    } else {
                        // Last page - validate all words and proceed
                        match validate_all_words(state) {
                            Ok(()) => {
                                state.error = None;
                                if let Some(next) = state.next_step_for_role() {
                                    state.advance_to_step(next);
                                }
                            }
                            Err(e) => {
                                state.error = Some(e);
                                // Reset current page selections and regenerate choices
                                let page_size = 6;
                                let current_page = state.seed_verification_state.current_page;

                                // Determine total word count
                                let total_words = if let Some(ref mnemonic) = state.mnemonic {
                                    let mnemonic_str = mnemonic.to_string();
                                    mnemonic_str.split_whitespace().count()
                                } else {
                                    12 // Fallback
                                };

                                let start_idx = current_page * page_size;
                                let end_idx = ((current_page + 1) * page_size).min(total_words);
                                let indices = &state.seed_verification_state.verification_indices
                                    [start_idx..end_idx];

                                for &word_idx in indices {
                                    state
                                        .seed_verification_state
                                        .selected_words
                                        .remove(&word_idx);
                                }

                                if let Err(err) = initialize_verification_state(state) {
                                    state.error = Some(err);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(e);
                    // Reset current page selections only
                    let page_size = 6;
                    let current_page = state.seed_verification_state.current_page;

                    // Determine total word count
                    let total_words = if let Some(ref mnemonic) = state.mnemonic {
                        let mnemonic_str = mnemonic.to_string();
                        mnemonic_str.split_whitespace().count()
                    } else {
                        12 // Fallback
                    };

                    let start_idx = current_page * page_size;
                    let end_idx = ((current_page + 1) * page_size).min(total_words);
                    let indices =
                        &state.seed_verification_state.verification_indices[start_idx..end_idx];

                    for &word_idx in indices {
                        state
                            .seed_verification_state
                            .selected_words
                            .remove(&word_idx);
                    }
                }
            }
        }

        ui.add_space(Spacing::MD);
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Set PIN step
pub fn render_set_pin(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.vertical_centered(|ui| {
        ui.heading("Set PIN");
        ui.add_space(Spacing::LG);
    });

    let mut callback = None;
    let pin_set = render_pin_setup(ui, &mut state.pin_setup_state, &mut callback);

    if pin_set {
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    ui.vertical_centered(|ui| {
        if button(ui, "← Back", ButtonStyle::Text).clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Main device: Scan/enter co-owner's keys
pub fn render_scan_coowner_keys(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    // Different heading and instructions based on device role
    match state.device_role {
        DeviceRole::SingleDeviceSeedHW => {
            ui.heading("Scan Hardware Wallet Keys");
            ui.add_space(Spacing::MD);
            ui.label("Scan your hardware wallet's account QR code to add it as co-owner.");
        }
        DeviceRole::SingleDeviceHWHW => {
            if !state.scanning_second_hw && state.first_hw_keys.is_none() {
                ui.heading("Scan First Hardware Wallet");
                ui.add_space(Spacing::MD);
                ui.label(
                    "Scan the first hardware wallet's account QR code (this will be the owner).",
                );
            } else {
                ui.heading("Scan Second Hardware Wallet");
                ui.add_space(Spacing::MD);
                ui.label("Scan the second hardware wallet's account QR code (this will be the co-owner).");
            }
        }
        _ => {
            ui.heading("Get Co-owner's Keys");
            ui.add_space(Spacing::MD);
            ui.label("First, have your co-owner complete their setup and share their keys.");
            ui.label("Then scan the QR code from their device or paste the key data.");
            egui::CollapsingHeader::new("How it works")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("1. Generate Keys: Your co-owner generates their public keys on their device.");
                    ui.label("2. Display QR Code: The co-owner displays a QR code with their keys.");
                    ui.label("3. Scan QR Code: Scan this QR code with your main device.");
                    ui.label("4. Complete Setup: After scanning, you'll proceed to complete the vault setup.");
                });
        }
    }
    ui.add_space(Spacing::MD);

    // Check if hardware wallet mode is active
    let hw_mode_active = !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
        || state.hw_batch_qr_scanner_state.pending_file_selection
        || state.hw_batch_qr_scanner_state.is_scanning
        || state.selected_hw_type.is_some();

    // Toggle between seed phrase and hardware wallet modes (only for regular 2-device setup)
    if state.device_role != DeviceRole::SingleDeviceSeedHW
        && state.device_role != DeviceRole::SingleDeviceHWHW
    {
        ui.horizontal(|ui| {
            ui.label("Co-owner type:");
            let seed_selected = !hw_mode_active;
            let hw_selected = hw_mode_active;

            if ui.selectable_label(seed_selected, "Seed Phrase").clicked() && hw_mode_active {
                // Switch to seed phrase mode
                state.hw_batch_qr_scanner_state.reset();
                state.selected_hw_type = None;
                state.coowner_keys = None;
            }
            if ui
                .selectable_label(hw_selected, "Hardware Wallet")
                .clicked()
                && !hw_mode_active
            {
                // Switch to hardware wallet mode
                if state.is_scanning_qr {
                    if let Some(ref mut camera) = state.camera_capture {
                        camera.stop_capture();
                    }
                    state.is_scanning_qr = false;
                }
                state.hw_batch_qr_scanner_state =
                    crate::ui::hardware_wallet::BatchQrScannerState::default();
            }
        });
    }

    if hw_mode_active {
        // Hardware wallet scanning mode
        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::SM);

        ui.heading("Scan Hardware Wallet QR");

        // Hardware wallet type selection (using consistent helper)
        render_hardware_wallet_type_selection(ui, state, "hw_type_selection_coowner");

        // Device-specific instructions (expandable)
        if let Some(hw_type) = state.selected_hw_type {
            egui::CollapsingHeader::new("Instructions")
                .default_open(false)
                .show(ui, |ui| {
                    for line in hw_type.instructions() {
                        ui.label(*line);
                    }
                });
        }

        // Show progress
        if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty() {
            ui.label(format!(
                "Scanned {} part(s)",
                state.hw_batch_qr_scanner_state.scanned_parts.len()
            ));
            ui.add_space(Spacing::SM);
        }

        // File selection for QR code image (only enabled if hardware wallet type is selected)
        let can_scan = state.selected_hw_type.is_some();
        if can_scan {
            if button(ui, "Select QR Code Image File", ButtonStyle::Secondary).clicked() {
                state.hw_batch_qr_scanner_state.pending_file_selection = true;
            }
        } else {
            // Show disabled button appearance
            ui.add_enabled(false, egui::Button::new("Select QR Code Image File"));
            ui.label(
                egui::RichText::new("Select a hardware wallet type above to enable scanning")
                    .weak(),
            );
        }

        // Handle file selection
        if state.hw_batch_qr_scanner_state.pending_file_selection && can_scan {
            state.hw_batch_qr_scanner_state.pending_file_selection = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.hw_batch_qr_scanner_state.selected_file = Some(path);
            }
        }

        // Show selected file and scan button
        if let Some(ref file_path) = state.hw_batch_qr_scanner_state.selected_file {
            ui.add_space(Spacing::SM);
            ui.label(format!("Selected: {}", file_path.display()));
            ui.add_space(Spacing::XS);

            let file_path_clone = file_path.clone();
            if button(ui, "Scan QR Code from File", ButtonStyle::Primary).clicked() {
                match crate::utils::qr::decode_qr_from_file(&file_path_clone) {
                    Ok(decoded) => {
                        match state
                            .hw_batch_qr_scanner_state
                            .process_scanned_part(decoded)
                        {
                            Ok(is_complete) => {
                                if is_complete {
                                    // Hardware wallet QR scanned successfully
                                    // Parse the decoded UR as AccountDescriptor and convert to CoownerKeys
                                    if let Some(ref decoded_base64) =
                                        state.hw_batch_qr_scanner_state.decoded_psbt
                                    {
                                        // The decoded_psbt field contains base64-encoded CBOR bytes from multi-part UR
                                        // Decode base64 and parse AccountDescriptor from CBOR bytes
                                        use base64::{engine::general_purpose, Engine as _};
                                        match general_purpose::STANDARD.decode(decoded_base64) {
                                            Ok(cbor_bytes) => {
                                                match bitvault_common::ur::parse_account_descriptor_from_cbor_bytes(&cbor_bytes) {
                                                    Ok(account_desc) => {
                                                        match bitvault_common::ur::convert_account_descriptor_to_coowner_keys(&account_desc) {
                                                            Ok(coowner_keys) => {
                                                                // For HW+HW single device, handle first vs second HW
                                                                if state.device_role == DeviceRole::SingleDeviceHWHW && !state.scanning_second_hw {
                                                                    // First HW scanned - store it and prepare for second HW
                                                                    state.first_hw_keys = Some(coowner_keys);
                                                                    state.first_hw_type = state.selected_hw_type; // Store first HW type
                                                                    state.scanning_second_hw = true;
                                                                    state.hw_batch_qr_scanner_state.reset(); // Reset scanner for second HW
                                                                    state.selected_hw_type = None; // Allow selecting different HW type for second
                                                                    state.error = None;
                                                                    // Stay on same step to scan second HW
                                                                } else {
                                                                    // Regular case or second HW in HW+HW mode
                                                                    state.coowner_keys = Some(coowner_keys);
                                                                    state.coowner_pubkeys = state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                                                    state.error = None;
                                                                    // Auto-advance to next step
                                                                    if let Some(next) = state.next_step_for_role() {
                                                                        state.advance_to_step(next);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!(
                                                                    "Failed to convert hardware wallet keys: {}\n\nThis may happen if:\n- The hardware wallet keys don't have the required derivation paths (m/48'/0'/0'/2'/0/0, etc.)\n- The QR code data is incomplete or corrupted\n\nPlease try scanning again or verify your hardware wallet is configured correctly.",
                                                                    e
                                                                );
                                                                state.error = Some(error_msg);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let error_msg = format!(
                                                            "Failed to parse hardware wallet AccountDescriptor: {}\n\nThe QR code may not be from a hardware wallet account export, or the data format is invalid.\n\nPlease ensure you're scanning the correct QR code from your hardware wallet.",
                                                            e
                                                        );
                                                        state.error = Some(error_msg);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                state.error = Some(format!(
                                                    "Failed to decode QR code data: {}\n\nPlease try scanning the QR code again.",
                                                    e
                                                ));
                                            }
                                        }
                                    } else {
                                        state.error = Some(
                                            "No decoded QR data available.\n\nThis may indicate the QR code scanning didn't complete properly. Please try scanning again.".to_string()
                                        );
                                    }
                                } else {
                                    // More parts needed - clear file selection for next scan
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                }
                            }
                            Err(e) => {
                                state.error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to decode QR code: {}", e));
                    }
                }
            }
        }

        // Show success message
        if state.hw_batch_qr_scanner_state.success {
            ui.add_space(Spacing::MD);
            ui.colored_label(
                egui::Color32::GREEN,
                "✓ Hardware wallet QR codes scanned successfully!",
            );
        }

        // Show errors (support multi-line)
        if let Some(ref error) = state.hw_batch_qr_scanner_state.error {
            ui.add_space(Spacing::SM);
            // Split error by newlines and display each line
            for line in error.lines() {
                ui.colored_label(egui::Color32::RED, line);
            }
        }

        // Show state.error as well (for conversion errors)
        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            // Display multi-line error messages
            for line in error.lines() {
                ui.colored_label(egui::Color32::RED, line);
            }
        }

        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::MD);

        // Continue button when hardware wallet QR is scanned
        if state.hw_batch_qr_scanner_state.success {
            // For HW+HW mode, check if we're waiting for second HW
            if state.device_role == DeviceRole::SingleDeviceHWHW
                && state.first_hw_keys.is_some()
                && state.coowner_keys.is_none()
            {
                // First HW scanned, waiting for second HW - scanner was reset
                ui.label(
                    egui::RichText::new(
                        "First hardware wallet scanned! Now scan the second hardware wallet above.",
                    )
                    .color(egui::Color32::GREEN),
                );
            } else {
                // Verify we have coowner keys from conversion (or first_hw_keys for HW+HW)
                let can_continue = state.coowner_keys.is_some()
                    || (state.device_role == DeviceRole::SingleDeviceHWHW
                        && state.first_hw_keys.is_some()
                        && state.coowner_keys.is_some());

                if can_continue {
                    let continue_response = button_large(ui, "Continue");
                    if continue_response.clicked() {
                        // Hardware wallet keys are ready
                        state.error = None;
                        if let Some(next) = state.next_step_for_role() {
                            state.advance_to_step(next);
                        }
                    }
                } else {
                    // Show processing state or error
                    ui.spinner();
                    ui.label("Processing hardware wallet keys...");
                    if state.error.is_some() {
                        ui.add_space(Spacing::SM);
                        ui.label("If this takes too long, try scanning again.");
                    }
                }
            }
        } else if state.device_role != DeviceRole::SingleDeviceSeedHW
            && state.device_role != DeviceRole::SingleDeviceHWHW
            && button(ui, "← Use Seed Phrase Co-owner Instead", ButtonStyle::Text).clicked()
        {
            // Option to switch back to seed phrase mode (only for regular 2-device setup)
            state.hw_batch_qr_scanner_state.reset();
            state.selected_hw_type = None;
            state.coowner_keys = None;
            state.error = None;
        }

        // Don't show seed phrase scanning UI in hardware wallet mode
        return;
    }

    ui.add_space(Spacing::LG);

    // Webcam scanning option - centered (seed phrase co-owner mode)
    ui.vertical_centered(|ui| {
        if state.is_scanning_qr {
            if button(ui, "Stop Scanning", ButtonStyle::Secondary).clicked() {
                if let Some(ref mut camera) = state.camera_capture {
                    camera.stop_capture();
                }
                state.is_scanning_qr = false;
            }
        } else if button(ui, "📷 Scan QR Code", ButtonStyle::Secondary).clicked() {
            let mut camera = crate::utils::camera::CameraCapture::new();
            match camera.start_capture() {
                Ok(()) => {
                    state.camera_capture = Some(camera);
                    state.is_scanning_qr = true;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to start camera: {}", e));
                }
            }
        }
    });

    // Show camera preview and scan
    if state.is_scanning_qr {
        if let Some(ref mut camera) = state.camera_capture {
            if let Some(texture) = camera.capture_frame(ctx) {
                ui.add_space(Spacing::MD);
                ui.image((texture.id(), egui::Vec2::new(400.0, 300.0)));
                ui.add_space(Spacing::SM);
                ui.label("Point camera at QR code");

                // Try to scan QR from current frame
                match camera.scan_qr_from_frame() {
                    Ok(qr_data) => {
                        // Validate it's co-owner keys
                        match bitvault_common::ur::decode_qr_data::<
                            bitvault_common::derivation::CoownerKeys,
                        >(&qr_data)
                        {
                            Ok(keys) => {
                                state.coowner_pubkeys = qr_data;
                                state.coowner_keys = Some(keys);
                                camera.stop_capture();
                                state.camera_capture = None;
                                state.is_scanning_qr = false;
                                state.error = None;
                                // Auto-advance to next step
                                if let Some(next) = state.next_step_for_role() {
                                    state.advance_to_step(next);
                                }
                                return;
                            }
                            Err(_) => {
                                // Not valid co-owner keys, keep scanning
                            }
                        }
                    }
                    Err(_) => {
                        // No QR code detected yet, keep scanning
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    ui.separator();
    ui.add_space(Spacing::MD);

    ui.label("Or paste co-owner's key data:");
    ui.add_space(Spacing::SM);

    let coowner_keys_response = ui.add(
        egui::TextEdit::multiline(&mut state.coowner_pubkeys)
            .hint_text("Paste the key data here...")
            .desired_width(400.0)
            .desired_rows(4),
    );

    // Auto-focus on step change
    if state.step_just_changed(VaultCreationStep::ScanCoownerKeys) {
        coowner_keys_response.request_focus();
    }

    ui.add_space(Spacing::MD);

    // Or load from file (try encrypted first, then plain JSON for backward compatibility)
    if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    // Try to parse as encrypted file first
                    match serde_json::from_str::<key_exchange::EncryptedCoownerKeysFile>(&contents)
                    {
                        Ok(encrypted_file) => {
                            // Decrypt
                            match key_exchange::decrypt_coowner_keys(&encrypted_file) {
                                Ok(coowner_keys) => {
                                    // Extract signature public key for File 2 encryption
                                    match general_purpose::STANDARD
                                        .decode(&encrypted_file.sender_public_key)
                                    {
                                        Ok(pubkey_bytes) => {
                                            match secp256k1::PublicKey::from_slice(&pubkey_bytes) {
                                                Ok(pubkey) => {
                                                    state.recipient_public_key = Some(pubkey);
                                                }
                                                Err(e) => {
                                                    state.error = Some(format!(
                                                        "Invalid public key in file: {}",
                                                        e
                                                    ));
                                                    return;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            state.error =
                                                Some(format!("Failed to decode public key: {}", e));
                                            return;
                                        }
                                    }

                                    // Encode decrypted keys back to JSON for display/QR
                                    match bitvault_common::ur::encode_qr_data(&coowner_keys) {
                                        Ok(keys_text) => {
                                            state.coowner_pubkeys = keys_text;
                                            state.coowner_keys = Some(coowner_keys);
                                            state.error = None;
                                            state.saved_key_file = Some(path);
                                        }
                                        Err(e) => {
                                            state.error = Some(format!(
                                                "Failed to encode decrypted keys: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.error = Some(format!("Failed to decrypt file: {}", e));
                                }
                            }
                        }
                        Err(_) => {
                            // Not encrypted, try plain JSON (backward compatibility)
                            state.coowner_pubkeys = contents.trim().to_string();
                            state.error = None;
                            state.saved_key_file = Some(path);
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", e));
                }
            }
        }
    }

    // Show warning if file was loaded
    if state.saved_key_file.is_some() && !state.coowner_pubkeys.is_empty() {
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: Delete the key file after successful vault creation.",
        );
    }

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    let continue_response = button_large(ui, "Continue");
    let continue_keyboard = continue_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if continue_response.clicked() || continue_keyboard {
        if state.coowner_pubkeys.trim().is_empty() {
            state.error = Some("Please scan, paste, or load the co-owner's key data".to_string());
        } else {
            // Try to decode as CoownerKeys (seed phrase co-owner)
            match bitvault_common::ur::decode_qr_data::<bitvault_common::derivation::CoownerKeys>(
                &state.coowner_pubkeys,
            ) {
                Ok(keys) => {
                    // Valid CoownerKeys format (seed phrase co-owner)
                    state.coowner_keys = Some(keys);
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(_) => {
                    // Not CoownerKeys format - might be hardware wallet UR format
                    // Check if it starts with "ur:" (UR format)
                    if state.coowner_pubkeys.trim().starts_with("ur:") {
                        // Hardware wallet UR format - backend will parse AccountDescriptor
                        // For now, mark coowner_keys as None - backend will handle conversion
                        state.coowner_keys = None;
                        state.error = None;
                        if let Some(next) = state.next_step_for_role() {
                            state.advance_to_step(next);
                        }
                    } else {
                        state.error = Some("Invalid key data format. Expected CoownerKeys JSON or Hardware Wallet UR format.".to_string());
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        // Stop camera if scanning
        if state.is_scanning_qr {
            if let Some(ref mut camera) = state.camera_capture {
                camera.stop_capture();
            }
            state.is_scanning_qr = false;
        }
        state.go_to_previous_step();
    }
}

/// Co-owner device: Display own keys for main device
pub fn render_display_own_keys(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Share Your Keys");
    ui.add_space(Spacing::SM);

    ui.label("Share this with the main device to link as co-owner.");
    ui.add_space(Spacing::MD);

    // Generate keys text if not already done
    if state.my_keys_text.is_none() {
        if let Some(ref mnemonic) = state.mnemonic {
            match bitvault_common::derivation::get_owner_keys(mnemonic) {
                Ok(owner_keys) => match bitvault_common::ur::encode_qr_data(&owner_keys) {
                    Ok(keys_text) => {
                        state.my_keys_text = Some(keys_text);
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to encode keys: {}", e));
                    }
                },
                Err(e) => {
                    state.error = Some(format!("Failed to derive keys: {}", e));
                }
            }
        }
    }

    if let Some(ref keys_text) = state.my_keys_text {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, keys_text) {
            ui.image((qr_texture.id(), egui::Vec2::new(280.0, 280.0)));
            ui.add_space(Spacing::SM);
        }

        // Copy and Save buttons on same row, centered
        let mut save_clicked = false;
        ui.horizontal(|ui| {
            // Calculate centering offset
            let button_width = 140.0 * 2.0 + Spacing::XS; // Two buttons + spacing
            let available = ui.available_width();
            if available > button_width {
                ui.add_space((available - button_width) / 2.0);
            }
            if button(ui, "Copy", ButtonStyle::Secondary).clicked() {
                ui.ctx().copy_text(keys_text.clone());
            }
            ui.add_space(Spacing::XS);
            if button(ui, "Save to File", ButtonStyle::Secondary).clicked() {
                save_clicked = true;
            }
        });

        // Handle save outside the horizontal block
        if save_clicked {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("coowner_keys.txt")
                .save_file()
            {
                // Parse keys from text
                match bitvault_common::ur::decode_qr_data::<bitvault_common::derivation::CoownerKeys>(
                    keys_text,
                ) {
                    Ok(coowner_keys) => {
                        // Encrypt and sign (generate signing key if not already stored)
                        let signing_key_opt = state.signing_secret_key.as_ref();
                        match key_exchange::encrypt_coowner_keys(&coowner_keys, signing_key_opt) {
                            Ok((encrypted_file, signing_key)) => {
                                // Store signing key for File 2 decryption
                                state.signing_secret_key = Some(signing_key);

                                // Serialize encrypted file to JSON
                                match serde_json::to_string_pretty(&encrypted_file) {
                                    Ok(json) => match std::fs::write(&path, json) {
                                        Ok(()) => {
                                            state.saved_key_file = Some(path.clone());
                                            state.error = None;
                                        }
                                        Err(e) => {
                                            state.error =
                                                Some(format!("Failed to save file: {}", e));
                                        }
                                    },
                                    Err(e) => {
                                        state.error = Some(format!(
                                            "Failed to serialize encrypted file: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                state.error = Some(format!("Failed to encrypt keys: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to parse keys: {}", e));
                    }
                }
            }
        }

        // Offer secure deletion if file was saved (compact inline)
        if let Some(ref file_path) = state.saved_key_file {
            if file_path.exists() {
                ui.add_space(Spacing::XS);
                let file_path_clone = file_path.clone();
                ui.horizontal(|ui| {
                    ui.label("✓ Saved");
                    if button(ui, "Delete", ButtonStyle::Danger).clicked() {
                        match bitvault_common::secure_delete_file(&file_path_clone)
                            .map_err(|e| e.to_string())
                        {
                            Ok(()) => {
                                state.saved_key_file = None;
                            }
                            Err(e) => {
                                state.error = Some(format!("Failed to delete: {}", e));
                            }
                        }
                    }
                });
            }
        }

        // Security warning (more compact)
        ui.add_space(Spacing::XS);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Delete file after sharing",
        );
    }

    ui.add_space(Spacing::MD);

    if button_large(ui, "I've Shared My Keys").clicked() {
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::SM);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Co-owner device: Enter exchange data from main device
pub fn render_enter_exchange_data(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Enter Vault Configuration");
    ui.add_space(Spacing::MD);

    ui.label("The main device owner will share the vault configuration with you");
    ui.label("after they create the vault. Scan the QR code or paste it below.");
    ui.add_space(Spacing::LG);

    // Webcam scanning option
    ui.horizontal(|ui| {
        if state.is_scanning_qr {
            if button(ui, "Stop Scanning", ButtonStyle::Secondary).clicked() {
                if let Some(ref mut camera) = state.camera_capture {
                    camera.stop_capture();
                }
                state.is_scanning_qr = false;
            }
        } else if button(ui, "📷 Scan QR Code", ButtonStyle::Secondary).clicked() {
            let mut camera = crate::utils::camera::CameraCapture::new();
            match camera.start_capture() {
                Ok(()) => {
                    state.camera_capture = Some(camera);
                    state.is_scanning_qr = true;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to start camera: {}", e));
                }
            }
        }
    });

    // Show camera preview and scan
    if state.is_scanning_qr {
        if let Some(ref mut camera) = state.camera_capture {
            if let Some(texture) = camera.capture_frame(ctx) {
                ui.add_space(Spacing::MD);
                ui.image((texture.id(), egui::Vec2::new(400.0, 300.0)));
                ui.add_space(Spacing::SM);
                ui.label("Point camera at QR code");

                // Try to scan QR from current frame
                match camera.scan_qr_from_frame() {
                    Ok(qr_data) => {
                        // Validate it's exchange data
                        match bitvault_common::ur::decode_qr_data::<
                            bitvault_common::ur::QrExchangeData,
                        >(&qr_data)
                        {
                            Ok(_) => {
                                state.exchange_data_input = qr_data;
                                camera.stop_capture();
                                state.camera_capture = None;
                                state.is_scanning_qr = false;
                                state.error = None;
                                // Auto-validate and continue
                                if let Some(next) = state.next_step_for_role() {
                                    state.advance_to_step(next);
                                }
                                return;
                            }
                            Err(_) => {
                                // Not valid exchange data, keep scanning
                            }
                        }
                    }
                    Err(_) => {
                        // No QR code detected yet, keep scanning
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    ui.separator();
    ui.add_space(Spacing::MD);

    ui.label("Or paste vault configuration:");
    ui.add_space(Spacing::SM);

    let exchange_data_response = ui.add(
        egui::TextEdit::multiline(&mut state.exchange_data_input)
            .hint_text("Paste the configuration data here...")
            .desired_width(400.0)
            .desired_rows(4),
    );

    // Auto-focus on step change
    if state.step_just_changed(VaultCreationStep::EnterExchangeData) {
        exchange_data_response.request_focus();
    }

    ui.add_space(Spacing::MD);

    if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    // Try to parse as encrypted file first
                    match serde_json::from_str::<key_exchange::EncryptedExchangeDataFile>(&contents)
                    {
                        Ok(encrypted_file) => {
                            // Decrypt using co-owner's signing private key
                            if let Some(ref signing_key) = state.signing_secret_key {
                                match key_exchange::decrypt_exchange_data(
                                    &encrypted_file,
                                    signing_key,
                                ) {
                                    Ok(exchange_data) => {
                                        // Encode decrypted data back to JSON for display
                                        match bitvault_common::ur::encode_qr_data(&exchange_data) {
                                            Ok(exchange_data_text) => {
                                                state.exchange_data_input = exchange_data_text;
                                                state.error = None;
                                                state.saved_exchange_file = Some(path);
                                            }
                                            Err(e) => {
                                                state.error = Some(format!(
                                                    "Failed to encode decrypted data: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        state.error =
                                            Some(format!("Failed to decrypt file: {}", e));
                                    }
                                }
                            } else {
                                state.error = Some("Missing signing key - cannot decrypt file. Please restart the workflow.".to_string());
                            }
                        }
                        Err(_) => {
                            // Not encrypted, try plain JSON (backward compatibility)
                            state.exchange_data_input = contents.trim().to_string();
                            state.error = None;
                            state.saved_exchange_file = Some(path);
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", e));
                }
            }
        }
    }

    // Show warning if file was loaded
    if state.saved_exchange_file.is_some() && !state.exchange_data_input.is_empty() {
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: Delete the config file after successful vault creation.",
        );
    }

    ui.add_space(Spacing::XL);

    let continue_response = button_large(ui, "Continue");
    let continue_keyboard = continue_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if continue_response.clicked() || continue_keyboard {
        if state.exchange_data_input.trim().is_empty() {
            state.error = Some("Please paste or load the vault configuration".to_string());
        } else {
            // Validate the exchange data
            match bitvault_common::ur::decode_qr_data::<bitvault_common::ur::QrExchangeData>(
                &state.exchange_data_input,
            ) {
                Ok(exchange_data) => {
                    // Store the main device's keys
                    state.coowner_keys = Some(exchange_data.coowner_public_keys);
                    // Extract time delay from exchange data
                    let time_delay = bitvault_common::utils::blocks_to_time_delay(
                        exchange_data.timelock_in_blocks,
                    );
                    state.time_delay_days = time_delay.days;
                    state.time_delay_hours = time_delay.hours;
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Invalid configuration data: {}", e));
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        // Stop camera if scanning
        if state.is_scanning_qr {
            if let Some(ref mut camera) = state.camera_capture {
                camera.stop_capture();
            }
            state.is_scanning_qr = false;
        }
        state.go_to_previous_step();
    }
}

/// Email authentication step
pub fn render_email_auth(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    ui.heading("Email Verification");
    ui.add_space(Spacing::MD);

    ui.label("Enter your email address to verify your identity:");
    ui.add_space(Spacing::MD);

    let email_response = ui.add(
        egui::TextEdit::singleline(&mut state.email)
            .hint_text("you@example.com")
            .desired_width(300.0)
            .margin(egui::vec2(8.0, 6.0)),
    );

    // Handle Enter key to send code
    let should_send_code = email_response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !state.code_sent;

    ui.add_space(Spacing::MD);

    if !state.code_sent {
        if button(ui, "Send Verification Code", ButtonStyle::Secondary).clicked()
            || should_send_code
        {
            if state.email.trim().is_empty() || !state.email.contains('@') {
                state.error = Some("Please enter a valid email address".to_string());
            } else if let Some(ref runtime) = app_state.runtime {
                // Check connectivity before network-dependent operation
                let is_online =
                    runtime.block_on(crate::services::network_check::check_connectivity());
                if !is_online {
                    state.error = Some(
                        "No internet connection. Please check your network and try again."
                            .to_string(),
                    );
                } else {
                    state.is_sending_code = true;
                    state.error = None;

                    let email = state.email.clone();
                    let network = app_state.network;
                    let result = runtime.block_on(async {
                        let temp_service = bitvault_common::wallet::VaultService::new(network);
                        temp_service.send_email_auth_code(&email).await
                    });

                    match result {
                        Ok(_) => {
                            state.code_sent = true;
                            state.is_sending_code = false;
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to send code: {}", e));
                            state.is_sending_code = false;
                        }
                    }
                }
            }
        }

        if state.is_sending_code {
            ui.spinner();
            ui.label("Sending...");
        }
    } else {
        ui.colored_label(egui::Color32::GREEN, "✓ Code sent! Check your email.");
        ui.add_space(Spacing::MD);

        ui.label("Enter the verification code:");
        ui.add_space(Spacing::SM);

        let auth_code_response = ui.add(
            egui::TextEdit::singleline(&mut state.auth_code)
                .hint_text("123456")
                .desired_width(150.0)
                .margin(egui::vec2(8.0, 6.0)),
        );

        // Auto-focus on step change (when code is sent)
        if state.step_just_changed(VaultCreationStep::EmailAuth) && state.code_sent {
            auth_code_response.request_focus();
        }

        // Handle Enter key to verify
        let should_verify =
            auth_code_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.add_space(Spacing::XL);

        let verify_response = button_large(ui, "Verify & Continue");
        let verify_keyboard = verify_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
        if verify_response.clicked() || should_verify || verify_keyboard {
            if state.auth_code.trim().is_empty() {
                state.error = Some("Please enter the verification code".to_string());
            } else {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.code_sent = false;
        state.auth_code.clear();
        state.go_to_previous_step();
    }
}

/// Create vault step
pub fn render_create_vault(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    let action_text = match state.device_role {
        DeviceRole::Main => "Create Vault",
        DeviceRole::Coowner => "Join Vault",
        DeviceRole::ViewOnly => "Set Up View-Only",
        DeviceRole::Restore => "Restore Vault",
        DeviceRole::SingleDeviceSeedHW => "Create Single Device Vault",
        DeviceRole::SingleDeviceHWHW => "Create Single Device Vault",
    };

    ui.heading(action_text);
    ui.add_space(Spacing::MD);

    if state.is_creating {
        ui.spinner();
        let status_text = match state.device_role {
            DeviceRole::Main => "Creating vault",
            DeviceRole::SingleDeviceSeedHW => "Creating single device vault",
            DeviceRole::SingleDeviceHWHW => "Creating single device vault",
            _ => "Joining vault",
        };
        ui.label(format!("{}...", status_text));
        return;
    }

    // Summary
    ui.label(format!("Vault Name: {}", state.vault_name));
    ui.label(format!(
        "Time Delay: {} days, {} hours",
        state.time_delay_days, state.time_delay_hours
    ));
    ui.label(format!("Email: {}", state.email));
    let role_display = match state.device_role {
        DeviceRole::Main => "Main Device",
        DeviceRole::Coowner => "Co-owner",
        DeviceRole::SingleDeviceSeedHW => "Single Device (Seed + HW)",
        DeviceRole::SingleDeviceHWHW => "Single Device (HW + HW)",
        _ => "Unknown",
    };
    ui.label(format!("Role: {}", role_display));

    ui.add_space(Spacing::XL);

    let create_response = button_large(ui, action_text);
    let create_keyboard = create_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if create_response.clicked() || create_keyboard {
        state.error = None;

        // Validation
        if state.vault_name.trim().is_empty() {
            state.error = Some("Vault name cannot be empty".to_string());
            state.is_creating = false;
            return;
        }

        // Validate email format (basic validation)
        let email = state.email.trim();
        if email.is_empty() {
            state.error = Some("Please enter an email address".to_string());
            state.is_creating = false;
            return;
        }
        if !email.contains('@') || !email.contains('.') || email.len() < 5 {
            state.error =
                Some("Please enter a valid email address (e.g., name@example.com)".to_string());
            state.is_creating = false;
            return;
        }
        // Check that @ is not at the start or end
        let at_pos = email.find('@').unwrap();
        if at_pos == 0 || at_pos == email.len() - 1 {
            state.error = Some("Please enter a valid email address".to_string());
            state.is_creating = false;
            return;
        }

        if state.auth_code.trim().is_empty() {
            state.error = Some("Please enter an authentication code".to_string());
            state.is_creating = false;
            return;
        }

        // Validate based on role
        match state.device_role {
            DeviceRole::SingleDeviceSeedHW => {
                // Need mnemonic and hardware wallet keys
                if state.mnemonic.is_none() {
                    state.error = Some("Seed phrase is required".to_string());
                    state.is_creating = false;
                    return;
                }
                if state.coowner_keys.is_none() && state.coowner_pubkeys.trim().is_empty() {
                    state.error = Some("Hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
            DeviceRole::SingleDeviceHWHW => {
                // Need both hardware wallets
                if state.first_hw_keys.is_none() {
                    state.error = Some("First hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
                if state.coowner_keys.is_none() && state.coowner_pubkeys.trim().is_empty() {
                    state.error = Some("Second hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
            _ => {
                // Regular 2-device setup
                if state.coowner_pubkeys.trim().is_empty() && state.coowner_keys.is_none() {
                    state.error = Some("Co-owner keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
        }

        // Create/join vault
        if let Some(runtime) = app_state.runtime.as_ref() {
            // Check connectivity before vault creation (network-dependent)
            let is_online = runtime.block_on(crate::services::network_check::check_connectivity());
            if !is_online {
                state.error = Some(
                    "No internet connection. Please check your network and try again.".to_string(),
                );
            } else {
                state.is_creating = true;
                let time_delay = TimeDelay {
                    days: state.time_delay_days,
                    hours: state.time_delay_hours,
                };
                let coowner_pubkeys = state.coowner_pubkeys.clone();
                let vault_name = state.vault_name.clone();
                let network = app_state.network;
                let email = state.email.clone();
                let auth_code = state.auth_code.clone();
                let runtime_handle = runtime.handle().clone();

                // Prepare data for single device vaults (validation outside async block)
                let hw_keys_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceSeedHW => {
                        // Convert hardware wallet keys to JSON string
                        if let Some(ref hw_keys) = state.coowner_keys {
                            match serde_json::to_string(hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize HW keys: {}", e));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else if !coowner_pubkeys.trim().is_empty() {
                            Some(coowner_pubkeys.clone())
                        } else {
                            None
                        }
                    }
                    DeviceRole::SingleDeviceHWHW => {
                        // Will be handled separately
                        None
                    }
                    _ => None,
                };

                let first_hw_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceHWHW => {
                        if let Some(ref first_hw_keys) = state.first_hw_keys {
                            match serde_json::to_string(first_hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize first HW keys: {}", e));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let second_hw_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceHWHW => {
                        if let Some(ref second_hw_keys) = state.coowner_keys {
                            match serde_json::to_string(second_hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize second HW keys: {}", e));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let result = runtime.block_on(async {
                    let mut vault_service = bitvault_common::wallet::VaultService::new(network);

                    match state.device_role {
                        DeviceRole::SingleDeviceSeedHW => {
                            let mnemonic = state.mnemonic.as_ref().unwrap(); // Already validated
                            let hw_keys_string = hw_keys_string_opt.as_ref().unwrap(); // Already validated
                            let hw_type_str = state
                                .selected_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            vault_service
                                .setup_single_device_vault_seed_hw(
                                    mnemonic,
                                    hw_keys_string,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                    &hw_type_str,
                                )
                                .await
                                .map(|_| (None, vault_service)) // Single device doesn't return QR data
                        }
                        DeviceRole::SingleDeviceHWHW => {
                            let first_hw_string = first_hw_string_opt.as_ref().unwrap(); // Already validated
                            let second_hw_string = second_hw_string_opt.as_ref().unwrap(); // Already validated
                            let first_hw_type_str = state
                                .first_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let second_hw_type_str = state
                                .selected_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            vault_service
                                .setup_single_device_vault_hw_hw(
                                    first_hw_string,
                                    second_hw_string,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                    &first_hw_type_str,
                                    &second_hw_type_str,
                                )
                                .await
                                .map(|_| (None, vault_service)) // Single device doesn't return QR data
                        }
                        _ => {
                            // Regular 2-device setup
                            let mnemonic = state.mnemonic.as_ref().unwrap(); // Already validated

                            let qr_result = vault_service
                                .setup_vault(
                                    mnemonic,
                                    &coowner_pubkeys,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                )
                                .await;

                            match qr_result {
                                Ok(qr) => Ok((Some(qr), vault_service)),
                                Err(e) => Err(e),
                            }
                        }
                    }
                });

                match result {
                    Ok((exchange_data, vault_service)) => {
                        // Exchange data is only returned for 2-device setups
                        if let Some(qr_data) = exchange_data {
                            state.exchange_data_output = Some(qr_data);
                        }

                        if let Err(e) = runtime_handle.block_on(async {
                            app_state.initialize_vault_from_service(vault_service).await
                        }) {
                            state.error = Some(format!("Failed to initialize vault: {}", e));
                            state.is_creating = false;
                            return;
                        }

                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        // Clear sensitive data (mnemonic) from memory now that vault is created
                        state.clear_sensitive_data();

                        state.is_creating = false;

                        // Single device vaults go directly to completed (no exchange data needed)
                        // Main device shows exchange data for 2-device setups, co-owner goes to completed
                        match state.device_role {
                            DeviceRole::Main => {
                                state.advance_to_step(VaultCreationStep::DisplayExchangeData);
                            }
                            DeviceRole::SingleDeviceSeedHW | DeviceRole::SingleDeviceHWHW => {
                                state.advance_to_step(VaultCreationStep::Completed);
                                navigation.navigate_to(View::Dashboard { tab: 0 });
                            }
                            _ => {
                                state.advance_to_step(VaultCreationStep::Completed);
                                navigation.navigate_to(View::Dashboard { tab: 0 });
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to create vault: {}", e));
                        state.is_creating = false;
                    }
                }
            }
        } else {
            state.error = Some("Missing mnemonic or runtime".to_string());
            state.is_creating = false;
        }
    }

    // Show Retry button when vault creation failed (e.g. network blip)
    if state.error.is_some() {
        ui.add_space(Spacing::MD);
        if button_large(ui, "Retry").clicked() {
            state.error = None;
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Main device: Display exchange data for co-owner
pub fn render_display_exchange_data(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Share with Co-owner");
    ui.add_space(Spacing::MD);

    ui.colored_label(egui::Color32::GREEN, "✓ Vault created successfully!");
    ui.add_space(Spacing::MD);

    ui.label("Share this configuration with your co-owner.");
    ui.label("They will enter it on their device to join the vault.");
    ui.add_space(Spacing::LG);

    if let Some(ref exchange_data) = state.exchange_data_output {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, exchange_data) {
            ui.image((qr_texture.id(), egui::Vec2::new(200.0, 200.0)));
            ui.add_space(Spacing::MD);
        }

        if button(ui, "Copy Configuration", ButtonStyle::Secondary).clicked() {
            ui.ctx().copy_text(exchange_data.clone());
        }

        ui.add_space(Spacing::SM);

        if button(ui, "Save to File", ButtonStyle::Secondary).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("vault_config.txt")
                .save_file()
            {
                // Parse exchange data from JSON
                match bitvault_common::ur::decode_qr_data::<bitvault_common::ur::QrExchangeData>(
                    exchange_data,
                ) {
                    Ok(exchange_data_struct) => {
                        // Encrypt with ECDH using co-owner's public key from File 1
                        if let Some(ref recipient_pubkey) = state.recipient_public_key {
                            match key_exchange::encrypt_exchange_data(
                                &exchange_data_struct,
                                recipient_pubkey,
                            ) {
                                Ok(encrypted_file) => {
                                    // Serialize encrypted file to JSON
                                    match serde_json::to_string_pretty(&encrypted_file) {
                                        Ok(json) => match std::fs::write(&path, json) {
                                            Ok(()) => {
                                                state.saved_exchange_file = Some(path.clone());
                                                state.error = None;
                                                ui.ctx().output_mut(|o| {
                                                    o.copied_text =
                                                        format!("Saved to: {}", path.display());
                                                });
                                            }
                                            Err(e) => {
                                                state.error =
                                                    Some(format!("Failed to save file: {}", e));
                                            }
                                        },
                                        Err(e) => {
                                            state.error = Some(format!(
                                                "Failed to serialize encrypted file: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to encrypt exchange data: {}", e));
                                }
                            }
                        } else {
                            // No recipient public key - save as plain JSON (backward compatibility)
                            match std::fs::write(&path, exchange_data) {
                                Ok(()) => {
                                    state.saved_exchange_file = Some(path.clone());
                                    state.error = None;
                                    ui.ctx().output_mut(|o| {
                                        o.copied_text = format!("Saved to: {}", path.display());
                                    });
                                }
                                Err(e) => {
                                    state.error = Some(format!("Failed to save file: {}", e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to parse exchange data: {}", e));
                    }
                }
            }
        }

        // Security warning
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: This file contains sensitive vault configuration. Delete it after use.",
        );

        // Offer secure deletion if file was saved
        if let Some(ref file_path) = state.saved_exchange_file {
            if file_path.exists() {
                ui.add_space(Spacing::SM);
                if button(ui, "🗑️ Delete Saved File", ButtonStyle::Danger).clicked() {
                    match bitvault_common::secure_delete_file(file_path).map_err(|e| e.to_string()) {
                        Ok(()) => {
                            state.saved_exchange_file = None;
                            ui.ctx().output_mut(|o| {
                                o.copied_text = "File securely deleted".to_string();
                            });
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to delete file: {}", e));
                        }
                    }
                }
            }
        }

        ui.add_space(Spacing::MD);

        ui.collapsing("Show Configuration Data", |ui| {
            ui.monospace(exchange_data);
        });
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Go to Dashboard").clicked() {
        state.advance_to_step(VaultCreationStep::Completed);
    }
}

/// Completed step
pub fn render_completed(
    ui: &mut egui::Ui,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("Vault Setup Complete!");
    ui.add_space(Spacing::LG);

    ui.colored_label(egui::Color32::GREEN, "✓ Your vault is ready to use.");
    ui.add_space(Spacing::MD);

    ui.label(format!("Vault Name: {}", state.vault_name));

    if let Some(ref address) = state.vault_address {
        ui.label(format!("Vault Address: {}", address));
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Go to Dashboard").clicked() {
        navigation.navigate_to(View::Dashboard { tab: 0 });
    }
}

// ============================================================================
// VIEW-ONLY FLOW
// ============================================================================

/// Scan descriptor QR for view-only mode
pub fn render_scan_descriptor_view_only(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    ui.label("Scan or paste the descriptor from your mobile device or hardware wallet.");
    ui.add_space(Spacing::SM);

    ui.colored_label(
        egui::Color32::from_rgb(100, 149, 237),
        "This will let you monitor your vault without signing capability.",
    );
    ui.add_space(Spacing::MD);

    // Option: Hardware Wallet or Mobile Device
    ui.horizontal(|ui| {
        ui.label("Source:");
        let hw_mode = !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
            || state.hw_batch_qr_scanner_state.pending_file_selection
            || state.hw_batch_qr_scanner_state.is_scanning;
        if ui.selectable_label(hw_mode, "Hardware Wallet").clicked() && !hw_mode {
            state.hw_batch_qr_scanner_state =
                crate::ui::hardware_wallet::BatchQrScannerState::default();
            state.import_descriptors_qr.clear();
        }
        if ui.selectable_label(!hw_mode, "Mobile Device").clicked() && hw_mode {
            state.hw_batch_qr_scanner_state.reset();
        }
    });

    if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
        || state.hw_batch_qr_scanner_state.pending_file_selection
        || state.hw_batch_qr_scanner_state.is_scanning
    {
        // Hardware wallet QR scanning mode
        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::SM);

        ui.heading("Scan Hardware Wallet Descriptor");

        // Hardware wallet type selection (using consistent helper)
        render_hardware_wallet_type_selection(ui, state, "hw_type_selection_view_only");

        ui.label("Scan QR code(s) from your hardware wallet:");
        ui.add_space(Spacing::SM);

        // Show progress
        if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty() {
            ui.label(format!(
                "Scanned {} part(s)",
                state.hw_batch_qr_scanner_state.scanned_parts.len()
            ));
            ui.add_space(Spacing::SM);
        }

        // File selection for QR code image (only enabled if hardware wallet type is selected)
        let can_scan = state.selected_hw_type.is_some();
        if can_scan {
            if button(ui, "Select QR Code Image File", ButtonStyle::Secondary).clicked() {
                state.hw_batch_qr_scanner_state.pending_file_selection = true;
            }
        } else {
            // Show disabled button appearance
            ui.add_enabled(false, egui::Button::new("Select QR Code Image File"));
            ui.label(
                egui::RichText::new("Select a hardware wallet type above to enable scanning")
                    .weak(),
            );
        }

        // Handle file selection
        if state.hw_batch_qr_scanner_state.pending_file_selection {
            state.hw_batch_qr_scanner_state.pending_file_selection = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.hw_batch_qr_scanner_state.selected_file = Some(path);
            }
        }

        // Show selected file and scan button
        if let Some(ref file_path) = state.hw_batch_qr_scanner_state.selected_file {
            ui.add_space(Spacing::SM);
            ui.label(format!("Selected: {}", file_path.display()));
            ui.add_space(Spacing::XS);

            let file_path_clone = file_path.clone();
            if button(ui, "Scan QR Code from File", ButtonStyle::Primary).clicked() {
                match crate::utils::qr::decode_qr_from_file(&file_path_clone) {
                    Ok(decoded) => {
                        match state
                            .hw_batch_qr_scanner_state
                            .process_scanned_part(decoded)
                        {
                            Ok(is_complete) => {
                                if is_complete {
                                    // Hardware wallet descriptor QR scanned successfully
                                    // Decode UR to get descriptor data
                                    if let Ok(Some(_message_bytes)) =
                                        bitvault_common::ur::decode_ur_part(
                                            &state
                                                .hw_batch_qr_scanner_state
                                                .scanned_parts
                                                .join("\n"),
                                        )
                                    {
                                        // Try to parse as AccountDescriptor
                                        match bitvault_common::ur::parse_crypto_account(
                                            &state.hw_batch_qr_scanner_state.scanned_parts[0],
                                        ) {
                                            Ok(_account_desc) => {
                                                // Store UR parts - backend will handle conversion
                                                state.import_descriptors_qr = state
                                                    .hw_batch_qr_scanner_state
                                                    .scanned_parts
                                                    .join("\n");
                                                state.hw_batch_qr_scanner_state.selected_file =
                                                    None;
                                                state.error = None;
                                            }
                                            Err(e) => {
                                                state.error = Some(format!("Failed to parse hardware wallet descriptor: {}", e));
                                            }
                                        }
                                    } else {
                                        // Store raw UR parts for backend processing
                                        state.import_descriptors_qr = state
                                            .hw_batch_qr_scanner_state
                                            .scanned_parts
                                            .join("\n");
                                        state.hw_batch_qr_scanner_state.selected_file = None;
                                        state.error = None;
                                    }
                                } else {
                                    // More parts needed
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                }
                            }
                            Err(e) => {
                                state.error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to decode QR code: {}", e));
                    }
                }
            }
        }

        // Show success/error
        if state.hw_batch_qr_scanner_state.success {
            ui.add_space(Spacing::SM);
            ui.colored_label(
                egui::Color32::GREEN,
                "✓ Hardware wallet descriptor scanned!",
            );
        }

        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::MD);
    }

    ui.add_space(Spacing::LG);

    ui.label("Paste the descriptor configuration:");
    ui.add_space(Spacing::SM);
    ui.add(
        egui::TextEdit::multiline(&mut state.import_descriptors_qr)
            .hint_text("Paste configuration from mobile app...")
            .desired_width(400.0)
            .desired_rows(3),
    );

    ui.add_space(Spacing::MD);

    // File load option
    ui.horizontal(|ui| {
        if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text/JSON", &["txt", "json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    state.import_descriptors_qr = contents;
                } else {
                    state.error = Some("Failed to read file".to_string());
                }
            }
        }
    });

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }
        state.error = None;
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// View-only setup complete
pub fn render_view_only_complete(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    if state.is_importing {
        ui.spinner();
        ui.label("Setting up view-only wallet...");
        return;
    }

    if state.vault_address.is_some() {
        // Success
        ui.colored_label(egui::Color32::GREEN, "✓ View-only wallet created!");
        ui.add_space(Spacing::MD);
        ui.label("You can now monitor your vault balance and transactions.");
        ui.label("Signing transactions will require your mobile device.");
        ui.add_space(Spacing::XL);

        if button_large(ui, "Open Wallet").clicked() {
            navigation.navigate_to(View::Dashboard { tab: 0 });
        }
    } else {
        // Setup button
        ui.label("Ready to set up view-only wallet?");
        ui.add_space(Spacing::LG);

        if button_large(ui, "Create View-Only Wallet").clicked() {
            state.is_importing = true;
            state.error = None;

            if let Some(ref runtime) = app_state.runtime {
                let descriptors_qr = state.import_descriptors_qr.clone();
                let vault_name = state.vault_name.clone();
                let network = app_state.network;
                let runtime_handle = runtime.handle().clone();

                // For view-only, we use a dummy mnemonic since we don't need signing
                let dummy_mnemonic = bitvault_common::generate_mnemonic(12)
                    .expect("Failed to generate dummy mnemonic");

                let result: Result<(bitvault_common::wallet::VaultService, String), String> =
                    runtime.block_on(async {
                        let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                        vault_service
                            .import_vault(
                                &dummy_mnemonic,
                                &descriptors_qr,
                                &vault_name,
                                false,
                                None,
                            )
                            .await
                            .map_err(|e| format!("Setup failed: {}", e))?;

                        let vault_address = vault_service
                            .get_address()
                            .map_err(|e| format!("Failed to get address: {}", e))?;
                        Ok((vault_service, vault_address))
                    });

                match result {
                    Ok((vault_service, vault_address)) => {
                        if let Err(e) = runtime_handle.block_on(async {
                            app_state.initialize_vault_from_service(vault_service).await
                        }) {
                            state.error = Some(format!("Failed to initialize: {}", e));
                            state.is_importing = false;
                            return;
                        }

                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        // Clear any sensitive data from memory
                        state.clear_sensitive_data();

                        state.vault_address = Some(vault_address);
                        state.is_importing = false;

                        // Navigate to dashboard
                        state.advance_to_step(VaultCreationStep::Completed);
                        navigation.navigate_to(View::Dashboard { tab: 0 });
                    }
                    Err(e) => {
                        state.error = Some(e);
                        state.is_importing = false;
                    }
                }
            } else {
                state.error = Some("Runtime not available".to_string());
                state.is_importing = false;
            }
        }
    }

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

// ============================================================================
// RESTORE FROM BACKUP FLOW
// ============================================================================

/// Select seed phrase size (12 or 24 words) before entering - matches mobile ImportSeedPhraseSizeView
pub fn render_select_seed_phrase_size(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    ui.label("How many words is your seed phrase?");
    ui.add_space(Spacing::LG);

    ui.vertical_centered(|ui| {
        if button_large(ui, "12 words").clicked() {
            state.import_seed_phrase_size = Some(12);
            state.error = None;
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }
        ui.add_space(Spacing::MD);
        if button_large(ui, "24 words").clicked() {
            state.import_seed_phrase_size = Some(24);
            state.error = None;
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }
    });

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Enter seed phrase from paper backup
pub fn render_enter_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    // Warning banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(80, 60, 0))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "⚠");
                ui.colored_label(
                    egui::Color32::WHITE,
                    "Only use your PAPER BACKUP - the seed phrase you wrote down during vault creation."
                );
            });
        });

    ui.add_space(Spacing::LG);

    let size = state.import_seed_phrase_size.unwrap_or(12);
    ui.label(format!("Enter your {} word seed phrase:", size));
    ui.add_space(Spacing::SM);

    ui.add(
        egui::TextEdit::multiline(&mut state.import_mnemonic_text)
            .hint_text("word1 word2 word3 word4 ...")
            .desired_width(400.0)
            .desired_rows(4)
            .password(true),
    ); // Hide for security

    ui.add_space(Spacing::SM);
    ui.label(
        egui::RichText::new("Your seed phrase is never transmitted and stays on this device.")
            .small()
            .color(egui::Color32::GRAY),
    );

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        let words: Vec<&str> = state.import_mnemonic_text.split_whitespace().collect();

        if words.is_empty() {
            state.error = Some("Please enter your seed phrase".to_string());
            return;
        }

        let expected = state.import_seed_phrase_size.unwrap_or(12);
        if words.len() != expected as usize {
            state.error = Some(format!(
                "Seed phrase should be {} words (you entered {})",
                expected,
                words.len()
            ));
            return;
        }

        // Validate mnemonic
        match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
            Ok(_) => {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
            Err(e) => {
                state.error = Some(format!("Invalid seed phrase: {}", e));
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Scan descriptor QR for restore flow
pub fn render_scan_descriptor_restore(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    ui.label("Now enter the descriptor configuration from your mobile device or hardware wallet.");
    ui.add_space(Spacing::SM);
    ui.label("On your mobile, go to Settings → Export Vault Descriptor.");
    ui.add_space(Spacing::MD);

    // Option: Hardware Wallet or Mobile Device
    ui.horizontal(|ui| {
        ui.label("Source:");
        let hw_mode = !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
            || state.hw_batch_qr_scanner_state.pending_file_selection
            || state.hw_batch_qr_scanner_state.is_scanning;
        if ui.selectable_label(hw_mode, "Hardware Wallet").clicked() && !hw_mode {
            state.hw_batch_qr_scanner_state =
                crate::ui::hardware_wallet::BatchQrScannerState::default();
            state.import_descriptors_qr.clear();
        }
        if ui.selectable_label(!hw_mode, "Mobile Device").clicked() && hw_mode {
            state.hw_batch_qr_scanner_state.reset();
        }
    });

    if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
        || state.hw_batch_qr_scanner_state.pending_file_selection
        || state.hw_batch_qr_scanner_state.is_scanning
    {
        // Hardware wallet QR scanning mode
        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::SM);

        ui.heading("Scan Hardware Wallet Descriptor");

        // Hardware wallet type selection (using consistent helper)
        render_hardware_wallet_type_selection(ui, state, "hw_type_selection_restore");

        ui.label("Scan QR code(s) from your hardware wallet:");
        ui.add_space(Spacing::SM);

        // Show progress
        if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty() {
            ui.label(format!(
                "Scanned {} part(s)",
                state.hw_batch_qr_scanner_state.scanned_parts.len()
            ));
            ui.add_space(Spacing::SM);
        }

        // File selection for QR code image (only enabled if hardware wallet type is selected)
        let can_scan = state.selected_hw_type.is_some();
        if can_scan {
            if button(ui, "Select QR Code Image File", ButtonStyle::Secondary).clicked() {
                state.hw_batch_qr_scanner_state.pending_file_selection = true;
            }
        } else {
            // Show disabled button appearance
            ui.add_enabled(false, egui::Button::new("Select QR Code Image File"));
            ui.label(
                egui::RichText::new("Select a hardware wallet type above to enable scanning")
                    .weak(),
            );
        }

        // Handle file selection
        if state.hw_batch_qr_scanner_state.pending_file_selection {
            state.hw_batch_qr_scanner_state.pending_file_selection = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.hw_batch_qr_scanner_state.selected_file = Some(path);
            }
        }

        // Show selected file and scan button
        if let Some(ref file_path) = state.hw_batch_qr_scanner_state.selected_file {
            ui.add_space(Spacing::SM);
            ui.label(format!("Selected: {}", file_path.display()));
            ui.add_space(Spacing::XS);

            let file_path_clone = file_path.clone();
            if button(ui, "Scan QR Code from File", ButtonStyle::Primary).clicked() {
                match crate::utils::qr::decode_qr_from_file(&file_path_clone) {
                    Ok(decoded) => {
                        match state
                            .hw_batch_qr_scanner_state
                            .process_scanned_part(decoded)
                        {
                            Ok(is_complete) => {
                                if is_complete {
                                    // Hardware wallet descriptor QR scanned successfully
                                    // Store UR parts for backend processing
                                    state.import_descriptors_qr =
                                        state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                } else {
                                    // More parts needed
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                }
                            }
                            Err(e) => {
                                state.error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to decode QR code: {}", e));
                    }
                }
            }
        }

        // Show success/error
        if state.hw_batch_qr_scanner_state.success {
            ui.add_space(Spacing::SM);
            ui.colored_label(
                egui::Color32::GREEN,
                "✓ Hardware wallet descriptor scanned!",
            );

            // If hardware wallet descriptor was scanned, ask if this is a single device vault
            ui.add_space(Spacing::MD);
            ui.separator();
            ui.add_space(Spacing::SM);
            ui.label("Is this a single device vault (Seed + Hardware Wallet)?");
            ui.label(egui::RichText::new("If your vault uses a seed phrase on this device plus a hardware wallet, select the hardware wallet type below.").small().weak());
            ui.add_space(Spacing::SM);

            // Hardware wallet type selection for single device detection
            render_hardware_wallet_type_selection(
                ui,
                state,
                "hw_type_selection_restore_single_device",
            );
        }

        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::MD);
    }

    ui.add_space(Spacing::LG);

    ui.label("Paste the descriptor configuration:");
    ui.add_space(Spacing::SM);
    ui.add(
        egui::TextEdit::multiline(&mut state.import_descriptors_qr)
            .hint_text("Paste configuration from mobile app...")
            .desired_width(400.0)
            .desired_rows(3),
    );

    ui.add_space(Spacing::MD);

    // File load option
    ui.horizontal(|ui| {
        if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text/JSON", &["txt", "json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    state.import_descriptors_qr = contents;
                } else {
                    state.error = Some("Failed to read file".to_string());
                }
            }
        }
    });

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if state.is_importing {
        ui.spinner();
        ui.label("Restoring vault...");
        return;
    }

    if button_large(ui, "Restore Vault").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }

        // Parse mnemonic
        let mnemonic =
            match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
                Ok(m) => m,
                Err(e) => {
                    state.error = Some(format!("Invalid seed phrase: {}", e));
                    return;
                }
            };

        state.is_importing = true;
        state.error = None;

        if let Some(ref runtime) = app_state.runtime {
            let descriptors_qr = state.import_descriptors_qr.clone();
            let vault_name = state.vault_name.clone();
            let network = app_state.network;
            let runtime_handle = runtime.handle().clone();

            // Determine if this is a single device vault (seed+HW) based on hardware wallet type selection
            let hw_type_opt =
                if state.hw_batch_qr_scanner_state.success && state.selected_hw_type.is_some() {
                    // Hardware wallet descriptor was scanned and type selected - likely single device vault
                    state.selected_hw_type.map(|t| t.title().to_string())
                } else {
                    None
                };

            let result: Result<(bitvault_common::wallet::VaultService, String), String> = runtime
                .block_on(async {
                    let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                    vault_service
                        .import_vault(
                            &mnemonic,
                            &descriptors_qr,
                            &vault_name,
                            false,
                            hw_type_opt.as_deref(),
                        )
                        .await
                        .map_err(|e| format!("Restore failed: {}", e))?;

                    let vault_address = vault_service
                        .get_address()
                        .map_err(|e| format!("Failed to get address: {}", e))?;
                    Ok((vault_service, vault_address))
                });

            match result {
                Ok((vault_service, vault_address)) => {
                    if let Err(e) = runtime_handle.block_on(async {
                        app_state.initialize_vault_from_service(vault_service).await
                    }) {
                        state.error = Some(format!("Failed to initialize: {}", e));
                        state.is_importing = false;
                        return;
                    }

                    if let Some(ref mut handler) = app_state.async_handler {
                        handler.fetch_balance();
                        handler.fetch_address();
                    }

                    // Clear sensitive data (seed phrase) from memory now that vault is restored
                    state.clear_sensitive_data();

                    state.vault_address = Some(vault_address);
                    state.is_importing = false;

                    // Go to PIN setup
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(e);
                    state.is_importing = false;
                }
            }
        } else {
            state.error = Some("Runtime not available".to_string());
            state.is_importing = false;
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}
