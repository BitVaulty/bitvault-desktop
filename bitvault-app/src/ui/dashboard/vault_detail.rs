//! Vault Detail Tab
//!
//! Shows:
//! - Balance (confirmed/available)
//! - Vault address
//! - Recent transactions
//! - Quick actions (Send, Receive buttons)

use crate::state::{AppState, Navigation};
use crate::ui::components::{card, badge, button, button_large, BadgeStyle, ButtonStyle, Colors, Spacing, Typography};
use chrono::{Local, TimeZone};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    let ctx = ui.ctx().clone();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::LG);

            if !app_state.is_vault_loaded() {
                card(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Spacing::XL);
                        ui.label(
                            Typography::heading("No Vault Loaded")
                                .color(Colors::text_primary(&ctx))
                        );
                        ui.add_space(Spacing::MD);
                        ui.label(
                            Typography::body("Create or import a vault to get started")
                                .color(Colors::text_secondary(&ctx))
                        );
                        ui.add_space(Spacing::LG);
                        if button_large(ui, "Create Vault").clicked() {
                            navigation.navigate_to(crate::state::View::VaultCreation);
                        }
                        ui.add_space(Spacing::XL);
                    });
                });
                return;
            }

            // Get vault data (read from shared state)
            let vault_data = match app_state.vault_data.lock() {
                Ok(data) => data.clone(),
                Err(_) => {
                    card(ui, |ui| {
                        ui.label(
                            Typography::body("Error: Mutex poisoned")
                                .color(Colors::ERROR)
                        );
                    });
                    return;
                }
            };

            // Balance Card - Large, prominent display
            card(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(Spacing::LG);
                    
                    // Main balance display
                    ui.vertical_centered(|ui| {
                        ui.label(
                            Typography::body("Total Balance")
                                .color(Colors::text_secondary(&ctx))
                        );
                        ui.add_space(Spacing::SM);
                        
                        let balance_text = vault_data.format_balance_btc();
                        ui.label(
                            Typography::heading_large(balance_text)
                                .color(Colors::text_primary(&ctx))
                        );
                        ui.label(
                            Typography::body("BTC")
                                .color(Colors::text_secondary(&ctx))
                        );
                    });
                    
                    ui.add_space(Spacing::MD);
                    ui.separator();
                    ui.add_space(Spacing::MD);
                    
                    // Confirmed vs Available in two-column layout
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                Typography::caption("Confirmed")
                                    .color(Colors::text_secondary(&ctx))
                            );
                            ui.label(
                                Typography::body(vault_data.format_balance_btc())
                                    .color(Colors::text_primary(&ctx))
                            );
                        });
                        
                        ui.add_space(Spacing::XL);
                        
                        ui.vertical(|ui| {
                            ui.label(
                                Typography::caption("Available")
                                    .color(Colors::text_secondary(&ctx))
                            );
                            ui.label(
                                Typography::body(vault_data.format_available_btc())
                                    .color(Colors::text_primary(&ctx))
                            );
                        });
                    });
                    
                    ui.add_space(Spacing::LG);
                });
            });

            ui.add_space(Spacing::MD);

            // Quick Actions - Large primary buttons (centered)
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if button_large(ui, "Send").clicked() {
                        navigation.navigate_to(crate::state::View::SendTransaction);
                    }
                    ui.add_space(Spacing::MD);
                    if button_large(ui, "Receive").clicked() {
                        navigation.navigate_to(crate::state::View::Receive);
                    }
                });
            });

            ui.add_space(Spacing::LG);

            // Vault Address Card
            card(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(Spacing::MD);
                    ui.label(
                        Typography::heading_small("Vault Address")
                            .color(Colors::text_primary(&ctx))
                    );
                    ui.add_space(Spacing::MD);
                    
                    if let Some(ref address) = vault_data.receive_address {
                        // Address in monospace with card background
                        ui.horizontal(|ui| {
                            let address_rect = ui.available_rect_before_wrap();
                            let (rect, response) = ui.allocate_exact_size(
                                egui::Vec2::new(address_rect.width(), 40.0),
                                egui::Sense::click()
                            );
                            
                            // Draw address background
                            ui.painter().rect_filled(
                                rect,
                                8.0,
                                if ctx.style().visuals.dark_mode {
                                    Colors::GRAY_900
                                } else {
                                    Colors::GRAY_100
                                },
                            );
                            ui.painter().rect_stroke(
                                rect,
                                8.0,
                                egui::Stroke::new(1.0, Colors::GRAY_300),
                            );
                            
                            // Draw address text
                            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                            let galley = ui.fonts(|f| f.layout_no_wrap(
                                address.clone(),
                                font_id,
                                Colors::text_primary(&ctx),
                            ));
                            ui.painter().galley(
                                rect.min + egui::Vec2::new(Spacing::MD, rect.height() / 2.0 - galley.size().y / 2.0),
                                galley,
                                Colors::text_primary(&ctx),
                            );
                            
                            // Click to copy
                            if response.clicked() {
                                ui.output_mut(|o| {
                                    o.copied_text = address.clone();
                                });
                            }
                        });
                        
                        ui.add_space(Spacing::MD);
                        ui.vertical_centered(|ui| {
                            if button(ui, "Copy Address", ButtonStyle::Secondary).clicked() {
                                ui.output_mut(|o| {
                                    o.copied_text = address.clone();
                                });
                            }
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.spinner();
                            ui.add_space(Spacing::SM);
                            ui.label(
                                Typography::body("Loading address...")
                                    .color(Colors::text_secondary(&ctx))
                            );
                        });
                        // Trigger async address fetch on first render
                        if !vault_data.is_loading {
                            if let Some(ref mut handler) = app_state.async_handler {
                                handler.fetch_address();
                            }
                        }
                    }
                    
                    ui.add_space(Spacing::MD);
                });
            });

            ui.add_space(Spacing::LG);

            // Secondary actions (centered)
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if button(ui, "Refresh", ButtonStyle::Secondary).clicked() || vault_data.needs_refresh() {
                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }
                    }
                    ui.add_space(Spacing::SM);
                    if button(ui, "Switch Vault", ButtonStyle::Text).clicked() {
                        navigation.navigate_to(crate::state::View::VaultSelection);
                    }
                });
            });

            ui.add_space(Spacing::LG);

        ui.add_space(20.0);

            // Recent Transactions Section
            ui.label(
                Typography::heading("Recent Transactions")
                    .color(Colors::text_primary(&ctx))
            );
            ui.add_space(Spacing::MD);

            // Fetch and display recent transactions
            if let (Some(vault_service), Some(runtime)) =
                (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
            {
                let result = runtime.block_on(async {
                    let vs = vault_service.read().await;
                    vs.list_transactions().await
                });

                match result {
                    Ok(transactions) => {
                        if transactions.is_empty() {
                            card(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(Spacing::LG);
                                    ui.label(
                                        Typography::body("No transactions yet")
                                            .color(Colors::text_secondary(&ctx))
                                    );
                                    ui.label(
                                        Typography::caption("When you make transactions, they will appear here")
                                            .color(Colors::text_muted(&ctx))
                                    );
                                    ui.add_space(Spacing::LG);
                                });
                            });
                        } else {
                            // Show up to 5 most recent transactions
                            let recent_txs: Vec<_> = transactions.iter().take(5).collect();

                            for tx in recent_txs {
                                let amount = tx.total_amount_btc();
                                let is_positive = amount >= 0.0;
                                
                                card(ui, |ui| {
                                    let response = ui.interact(
                                        ui.available_rect_before_wrap(),
                                        ui.id().with(tx.tx_id.clone()),
                                        egui::Sense::click(),
                                    );
                                    
                                    if response.hovered() {
                                        ui.painter().rect_filled(
                                            response.rect,
                                            12.0,
                                            if ctx.style().visuals.dark_mode {
                                                Colors::GRAY_700
                                            } else {
                                                Colors::GRAY_100
                                            },
                                        );
                                    }
                                    
                                    ui.horizontal(|ui| {
                                        // Status badge
                                        let status_badge = match tx.status {
                                            bitvault_common::types::TransactionStatus::Pending => {
                                                BadgeStyle::Warning
                                            }
                                            bitvault_common::types::TransactionStatus::Sent => {
                                                BadgeStyle::Error
                                            }
                                            bitvault_common::types::TransactionStatus::Received => {
                                                BadgeStyle::Success
                                            }
                                        };
                                        let status_text = match tx.status {
                                            bitvault_common::types::TransactionStatus::Pending => "Pending",
                                            bitvault_common::types::TransactionStatus::Sent => "Sent",
                                            bitvault_common::types::TransactionStatus::Received => "Received",
                                        };
                                        badge(ui, status_text, status_badge);
                                        
                                        ui.add_space(Spacing::MD);
                                        
                                        // Amount
                                        let amount_str = if is_positive {
                                            format!("+{:.8} BTC", amount)
                                        } else {
                                            format!("{:.8} BTC", amount.abs())
                                        };
                                        ui.label(
                                            Typography::body(amount_str)
                                                .color(if is_positive { Colors::SUCCESS } else { Colors::ERROR })
                                                .strong()
                                        );
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Date
                                            if tx.timestamp > 0 {
                                                if let Some(dt) = Local.timestamp_opt(tx.timestamp, 0).single() {
                                                    ui.label(
                                                        Typography::caption(dt.format("%b %d, %Y").to_string())
                                                            .color(Colors::text_secondary(&ctx))
                                                    );
                                                }
                                            }
                                            
                                            ui.add_space(Spacing::SM);
                                            
                                            // Arrow indicator
                                            ui.label(
                                                egui::RichText::new("→")
                                                    .color(Colors::text_secondary(&ctx))
                                                    .size(18.0)
                                            );
                                        });
                                    });
                                    
                                    if response.clicked() {
                                        navigation.navigate_to(
                                            crate::state::View::TransactionDetail {
                                                txid: tx.tx_id.clone(),
                                            },
                                        );
                                    }
                                });
                                ui.add_space(Spacing::SM);
                            }

                            if transactions.len() > 5 {
                                ui.add_space(Spacing::MD);
                                ui.vertical_centered(|ui| {
                                    if button(ui, "View All Transactions", ButtonStyle::Secondary).clicked() {
                                        navigation.set_dashboard_tab(1); // Switch to transaction history tab
                                    }
                                });
                            }
                        }
                    }
                    Err(e) => {
                        card(ui, |ui| {
                            ui.label(
                                Typography::body(format!("Failed to load transactions: {}", e))
                                    .color(Colors::ERROR)
                            );
                        });
                    }
                }
            } else {
                card(ui, |ui| {
                    ui.label(
                        Typography::body("Vault not loaded")
                            .color(Colors::text_secondary(&ctx))
                    );
                });
            }
            
            ui.add_space(Spacing::LG);
        });
    });
}

// Note: Async data fetching is implemented using AsyncCommandHandler
// which uses block_on for quick operations (balance, address fetching)
// This is acceptable for egui's immediate mode UI
