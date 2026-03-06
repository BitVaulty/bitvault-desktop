//! BitVault Desktop Application Library
//!
//! This library exposes the application modules for testing purposes.
//! The main entry point is still in main.rs

// VaultService in Arc is used from main thread only; Send/Sync not required for egui
#![allow(clippy::arc_with_non_send_sync)]
// Various API surfaces and future-use code
#![allow(dead_code)]

pub mod app;
pub mod models;
pub mod services;
pub mod settings;
pub mod state;
pub mod ui;
pub mod utils;
