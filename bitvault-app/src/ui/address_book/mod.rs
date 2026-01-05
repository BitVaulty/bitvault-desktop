//! Address Book UI
//!
//! UI for viewing and managing saved addresses

pub mod entry;
pub mod list;

pub use list::{render_address_book, AddressBookState};
