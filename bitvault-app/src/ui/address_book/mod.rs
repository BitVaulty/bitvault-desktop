//! Address Book UI
//!
//! UI for viewing and managing saved addresses

pub mod list;
pub mod entry;

pub use list::{render_address_book, AddressBookState};
