//! Shared vault creation steps (role, name, time_delay, mnemonic, display_seed, verify, set_pin, completed)

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{button, button_large, ButtonStyle, Colors, Spacing};
use crate::ui::pin::render_pin_setup;
use crate::ui::vault_creation::{
    DeviceRole, VaultCreationState, VaultCreationStep,
};
use crate::utils::icons::{icon_image, Icon};
use bip39::Mnemonic;
use eframe::egui;
use std::collections::HashSet;

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
                    state.error = Some(format!("Failed to generate mnemonic: {}", crate::utils::sanitize_error_for_ui(&e)));
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

