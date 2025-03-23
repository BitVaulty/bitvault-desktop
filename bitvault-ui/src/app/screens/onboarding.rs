use eframe::egui::{self, Color32, RichText, Ui};
use std::sync::OnceLock;

use crate::app::assets;
use crate::app::state::{SharedAppState, View};

// Common function to render a centered onboarding container
fn render_container(ui: &mut Ui, render_content: impl FnOnce(&mut Ui)) {
    // Set the background to white
    let screen_rect = ui.max_rect();
    ui.painter().rect_filled(screen_rect, 0.0, Color32::WHITE);

    // Calculate the available space
    let available_width = screen_rect.width();
    let available_height = screen_rect.height();

    // Content width (fixed at 328px for mobile designs)
    let content_width: f32 = 328.0;
    let content_height: f32 = 650.0; // Approximate height of content

    // Calculate vertical padding to center content
    let min_padding: f32 = 10.0;
    let vertical_padding = (available_height - content_height).max(min_padding) / 2.0;

    // Add container that centers content both horizontally and vertically
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(vertical_padding);

                // Create a container with fixed width but centered horizontally
                let min_side_margin: f32 = 20.0;
                let container_width = content_width.min(available_width - min_side_margin);
                ui.allocate_ui_with_layout(
                    egui::vec2(container_width, content_height),
                    egui::Layout::top_down(egui::Align::Center),
                    render_content,
                );

                ui.add_space(vertical_padding);
            });
        });
}

// Helper function to draw arrow navigation indicators
fn draw_navigation_arrows(ui: &mut Ui, screen_number: usize, state: &SharedAppState) {
    // Available width needed for centering calculation
    let available_width = ui.available_width();

    // Create fixed-width dots
    let active_width = 15.0;
    let inactive_width = 5.0;
    let dot_height = 4.0; // Slightly thicker for better visibility while still bead-like
    let dot_spacing = 3.0;
    let click_padding = 12.0; // Larger click area padding for better usability

    // Calculate total width of all dots
    let total_dot_width = match screen_number {
        1 => active_width + 2.0 * inactive_width + 2.0 * dot_spacing,
        2 => inactive_width + active_width + inactive_width + 2.0 * dot_spacing,
        3 => 2.0 * inactive_width + active_width + 2.0 * dot_spacing,
        _ => active_width + 2.0 * inactive_width + 2.0 * dot_spacing,
    };

    // Add space for centering
    let left_padding = (available_width - total_dot_width) / 2.0;

    ui.horizontal(|ui| {
        ui.add_space(left_padding);

        // Create a container for our dots with extra height for easier clicking
        let response = ui.allocate_rect(
            egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(total_dot_width, dot_height + click_padding),
            ),
            egui::Sense::click(), // Make the entire area clickable
        );

        // Draw the dots directly using the painter
        let painter = ui.painter();
        let mut current_x = response.rect.min.x;
        let center_y = response.rect.center().y;

        // Store click positions for later processing
        let mut click_areas = Vec::new();

        // Draw all dots
        for i in 1..=3 {
            if i > 1 {
                current_x += dot_spacing;
            }

            // Determine dot properties based on state
            let (width, color) = if i == screen_number {
                (active_width, Color32::from_rgb(17, 165, 238))
            } else {
                (inactive_width, Color32::from_rgb(217, 217, 217))
            };

            // Calculate the dot rectangle
            let dot_rect = egui::Rect::from_min_size(
                egui::pos2(current_x, center_y - dot_height / 2.0),
                egui::vec2(width, dot_height),
            );

            // Draw the dot
            painter.rect_filled(dot_rect, dot_height / 2.0, color);

            // Store click area if this is an inactive dot
            if i != screen_number {
                // Create a larger clickable area
                let click_rect = egui::Rect::from_min_max(
                    egui::pos2(current_x - 2.0, center_y - (click_padding / 2.0)),
                    egui::pos2(current_x + width + 2.0, center_y + (click_padding / 2.0)),
                );

                click_areas.push((click_rect, i));
            }

            // Move to the next dot position
            current_x += width;
        }

        // Handle clicks for navigation
        if response.clicked() {
            if let Some(mouse_pos) = ui.ctx().pointer_latest_pos() {
                // Handle clicks directly on dots
                for (rect, idx) in click_areas {
                    if rect.contains(mouse_pos) {
                        if let Ok(mut app_state) = state.write() {
                            app_state.current_view = match idx {
                                1 => View::OnboardingOne,
                                2 => View::OnboardingTwo,
                                3 => View::OnboardingThree,
                                _ => View::OnboardingOne,
                            };
                        }
                        break;
                    }
                }
            }
        }
    });
}

pub fn render_one(ui: &mut Ui, state: &SharedAppState) {
    render_container(ui, |ui| {
        // Upper panel with illustration
        ui.add_space(80.0); // Status bar + top spacing

        // Illustration frame - use SVG
        ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
            ui.vertical_centered(|ui| {
                // Load and display the SVG image
                static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                let texture = TEXTURE_ID.get_or_init(|| {
                    log::debug!("Loading onboarding1.svg - this should only happen once");
                    assets::load_svg_as_texture(ui.ctx(), "onboarding1", "assets/onboarding1.svg")
                });

                if let Some(texture) = texture {
                    // Get texture size and available space
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();

                    // Scale to fit within the available space while preserving aspect ratio
                    let scale = (available_size.x / texture_size.x)
                        .min(available_size.y / texture_size.y)
                        .min(1.0); // Don't scale up if image is smaller

                    let display_size = texture_size * scale;

                    ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                    ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                } else {
                    ui.colored_label(Color32::RED, "Failed to load SVG image");

                    // Fallback to drawn elements if SVG fails to load
                    ui.add_space(20.0);

                    // Draw a shield with keys icon (for multisig)
                    let center = ui.available_rect_before_wrap().center();
                    let shield_size = 120.0;

                    // Shield background
                    ui.painter().circle_filled(
                        center,
                        shield_size / 2.0,
                        Color32::from_rgb(240, 240, 240),
                    );

                    // Shield border
                    ui.painter().circle_stroke(
                        center,
                        shield_size / 2.0 + 1.0,
                        egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                    );

                    // Draw three key symbols
                    let key_color = Color32::from_rgb(50, 50, 50);
                    let key_spacing = shield_size * 0.3;

                    // Draw three symbolic keys
                    for i in -1..=1 {
                        let key_center = center + egui::vec2(i as f32 * key_spacing, 0.0);

                        // Key head (circle)
                        ui.painter().circle_filled(
                            key_center - egui::vec2(0.0, shield_size * 0.15),
                            shield_size * 0.08,
                            key_color,
                        );

                        // Key shaft
                        let shaft_rect = egui::Rect::from_min_size(
                            key_center + egui::vec2(-shield_size * 0.03, -shield_size * 0.05),
                            egui::vec2(shield_size * 0.06, shield_size * 0.25),
                        );

                        ui.painter().rect_filled(shaft_rect, 2.0, key_color);

                        // Key teeth
                        let teeth_top = key_center.y + shield_size * 0.08;
                        let teeth_width = shield_size * 0.04;
                        let teeth_height = shield_size * 0.06;

                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(key_center.x - teeth_width / 2.0, teeth_top),
                                egui::vec2(teeth_width, teeth_height),
                            ),
                            1.0,
                            key_color,
                        );
                    }
                }
            });
        });

        // Content
        ui.add_space(32.0);
        ui.heading(
            RichText::new("Multisig security")
                .color(Color32::BLACK)
                .size(24.0),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Secure your funds with 2-of-3 multisig vaults. For extra security spread the 3 keys across 3 different geolocation and store an extra copy of one key in a physical vault or similar.")
            .color(Color32::from_rgb(82, 82, 82))
            .size(14.0)
        );

        // Indicators
        ui.add_space(24.0);
        draw_navigation_arrows(ui, 1, state);

        // Buttons at the bottom
        ui.add_space(32.0);

        if ui
            .add(
                egui::Button::new(
                    RichText::new("Create a new wallet")
                        .color(Color32::WHITE)
                        .size(16.0),
                )
                .min_size(egui::vec2(328.0, 48.0))
                .fill(Color32::BLACK)
                .rounding(16.0),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::OnboardingTwo;
            }
        }

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(
                    RichText::new("I already have a wallet")
                        .color(Color32::BLACK)
                        .size(16.0),
                )
                .min_size(egui::vec2(328.0, 48.0))
                .frame(false),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::Home;
                app_state.onboarding_completed = true;
            }
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0),
        );

        // Navigation hint
        ui.add_space(4.0);
        ui.label(
            RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0),
        );
    });
}

pub fn render_two(ui: &mut Ui, state: &SharedAppState) {
    render_container(ui, |ui| {
        // Upper panel with illustration
        ui.add_space(80.0); // Status bar + top spacing

        // Illustration frame - use SVG
        ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
            ui.vertical_centered(|ui| {
                // Load and display the SVG image
                static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                let texture = TEXTURE_ID.get_or_init(|| {
                    log::debug!("Loading onboarding2.svg - this should only happen once");
                    assets::load_svg_as_texture(ui.ctx(), "onboarding2", "assets/onboarding2.svg")
                });

                if let Some(texture) = texture {
                    // Get texture size and available space
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();

                    // Scale to fit within the available space while preserving aspect ratio
                    let scale = (available_size.x / texture_size.x)
                        .min(available_size.y / texture_size.y)
                        .min(1.0); // Don't scale up if image is smaller

                    let display_size = texture_size * scale;

                    ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                    ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                } else {
                    ui.colored_label(Color32::RED, "Failed to load SVG image");

                    // Fallback to drawn elements if SVG fails to load
                    ui.add_space(20.0);

                    // Draw a clock (for time delay)
                    let center = ui.available_rect_before_wrap().center();
                    let clock_size = 120.0;

                    // Clock face
                    ui.painter().circle_filled(
                        center,
                        clock_size / 2.0,
                        Color32::from_rgb(240, 240, 240),
                    );

                    // Clock border
                    ui.painter().circle_stroke(
                        center,
                        clock_size / 2.0,
                        egui::Stroke::new(2.0, Color32::from_rgb(50, 50, 50)),
                    );

                    // Clock hands
                    let hour_hand = center + egui::vec2(0.0, -clock_size * 0.25);
                    let minute_hand = center + egui::vec2(clock_size * 0.3, 0.0);

                    ui.painter()
                        .line_segment([center, hour_hand], egui::Stroke::new(3.0, Color32::BLACK));

                    ui.painter().line_segment(
                        [center, minute_hand],
                        egui::Stroke::new(3.0, Color32::BLACK),
                    );

                    // Clock center dot
                    ui.painter().circle_filled(center, 4.0, Color32::BLACK);
                }
            });
        });

        // Content
        ui.add_space(32.0);
        ui.heading(
            RichText::new("Time-delay protection")
                .color(Color32::BLACK)
                .size(28.0),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Set time-delays and prevent unauthorised withdrawals. The xPUB is of VITAL importance to recover your multisig vault. Keep AT LEAST a copy of the xPUB together with each key.")
            .color(Color32::from_rgb(82, 82, 82))
            .size(14.0)
        );

        // Indicators
        ui.add_space(24.0);
        draw_navigation_arrows(ui, 2, state);

        // Buttons at the bottom
        ui.add_space(32.0);

        if ui
            .add(
                egui::Button::new(RichText::new("Continue").color(Color32::WHITE).size(16.0))
                    .min_size(egui::vec2(328.0, 48.0))
                    .fill(Color32::BLACK)
                    .rounding(16.0),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::OnboardingThree;
            }
        }

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(RichText::new("Back").color(Color32::BLACK).size(16.0))
                    .min_size(egui::vec2(328.0, 48.0))
                    .frame(false),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::OnboardingOne;
            }
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0),
        );

        // Navigation hint
        ui.add_space(4.0);
        ui.label(
            RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0),
        );
    });
}

pub fn render_three(ui: &mut Ui, state: &SharedAppState) {
    render_container(ui, |ui| {
        // Upper panel with illustration
        ui.add_space(80.0); // Status bar + top spacing

        // Illustration frame - use SVG
        ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
            ui.vertical_centered(|ui| {
                // Load and display the SVG image
                static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                let texture = TEXTURE_ID.get_or_init(|| {
                    log::debug!("Loading onboarding3.svg - this should only happen once");
                    assets::load_svg_as_texture(ui.ctx(), "onboarding3", "assets/onboarding3.svg")
                });

                if let Some(texture) = texture {
                    // Get texture size and available space
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();

                    // Scale to fit within the available space while preserving aspect ratio
                    let scale = (available_size.x / texture_size.x)
                        .min(available_size.y / texture_size.y)
                        .min(1.0); // Don't scale up if image is smaller

                    let display_size = texture_size * scale;

                    ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                    ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                } else {
                    ui.colored_label(Color32::RED, "Failed to load SVG image");

                    // Fallback to the shield rendering if SVG fails
                    // Center position
                    let center = ui.min_rect().center();

                    // Draw a shield shape for the notification icon
                    let shield_size = 100.0;
                    let shield_radius = shield_size / 2.0;

                    // Draw shield background (light gray)
                    ui.painter().circle_filled(
                        center,
                        shield_radius,
                        Color32::from_rgb(245, 245, 245),
                    );

                    // Draw shield outline
                    ui.painter().circle_stroke(
                        center,
                        shield_radius,
                        egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                    );

                    // Draw lock icon inside the shield
                    let lock_size = 40.0;
                    let lock_top = center.y - lock_size * 0.2;
                    let lock_bottom = center.y + lock_size * 0.5;
                    let lock_left = center.x - lock_size * 0.3;
                    let lock_right = center.x + lock_size * 0.3;

                    // Lock body
                    let lock_body = egui::Rect::from_min_max(
                        egui::pos2(lock_left, lock_top),
                        egui::pos2(lock_right, lock_bottom),
                    );
                    ui.painter()
                        .rect_filled(lock_body, 5.0, Color32::from_rgb(30, 30, 30));

                    // Lock shackle (arc)
                    let shackle_radius = lock_size * 0.4;
                    let shackle_center = egui::pos2(center.x, lock_top - shackle_radius * 0.3);
                    let shackle_stroke = egui::Stroke::new(6.0, Color32::from_rgb(30, 30, 30));

                    // Draw a semi-circle for the shackle
                    ui.painter()
                        .circle_stroke(shackle_center, shackle_radius, shackle_stroke);
                }
            });
        });

        // Content
        ui.add_space(32.0);
        ui.heading(
            RichText::new("Secret notifications")
                .color(Color32::BLACK)
                .size(28.0),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Stay informed about important wallet events and security updates. Secret notifications are end-to-end encrypted to protect your privacy and security.")
            .color(Color32::from_rgb(82, 82, 82))
            .size(14.0)
        );

        // Indicators
        ui.add_space(24.0);
        draw_navigation_arrows(ui, 3, state);

        // Buttons at the bottom
        ui.add_space(32.0);

        if ui
            .add(
                egui::Button::new(RichText::new("Let's go!").color(Color32::WHITE).size(16.0))
                    .min_size(egui::vec2(328.0, 48.0))
                    .fill(Color32::BLACK)
                    .rounding(16.0),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::Home;
                app_state.onboarding_completed = true;
            }
        }

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(RichText::new("Back").color(Color32::BLACK).size(16.0))
                    .min_size(egui::vec2(328.0, 48.0))
                    .frame(false),
            )
            .clicked()
        {
            if let Ok(mut app_state) = state.write() {
                app_state.current_view = View::OnboardingTwo;
            }
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0),
        );

        // Navigation hint
        ui.add_space(4.0);
        ui.label(
            RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0),
        );
    });
}
