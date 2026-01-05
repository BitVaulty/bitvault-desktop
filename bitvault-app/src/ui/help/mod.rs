//! Help and Support UI
//!
//! Provides FAQ and help information for users

mod question_detail;
mod question_list;

pub use question_list::{render_help_and_support, HelpAndSupportState};

/// Question and Answer data structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionAndAnswer {
    pub question: String,
    pub answer: String,
}

impl QuestionAndAnswer {
    pub fn all_questions() -> Vec<Self> {
        vec![
            Self {
                question: "What is Time-delay?".to_string(),
                answer: "Your vault's security time-delay is currently active. This protects your funds by enforcing a waiting period before the transaction completes.".to_string(),
            },
            Self {
                question: "How can I send BTC from my vault?".to_string(),
                answer: "To send BTC from your vault:\n1. Go to the Dashboard\n2. Click 'Send Transaction'\n3. Enter the recipient address and amount\n4. Review the transaction preview\n5. Sign and broadcast the transaction".to_string(),
            },
            Self {
                question: "How do I create a new vault?".to_string(),
                answer: "To create a new vault:\n1. Go to Vault Selection\n2. Click 'Create New Vault'\n3. Follow the setup wizard:\n   - Generate or import a seed phrase\n   - Set a time delay\n   - Set a PIN\n   - Link a co-owner\n4. Complete vault creation".to_string(),
            },
            Self {
                question: "What is a co-owner?".to_string(),
                answer: "A co-owner is a second device or person that shares control of your vault. Your vault uses a 2-of-3 multisig setup, requiring signatures from both you and your co-owner (or the convenience service) to complete transactions.".to_string(),
            },
            Self {
                question: "How do I backup my vault?".to_string(),
                answer: "You can backup your vault in several ways:\n1. Manual backup: Export vault metadata as a ZIP file\n2. pCloud backup: Sync your vault to pCloud\n3. Write down your seed phrase: Keep it in a safe place\n\nAlways keep multiple backups in different locations.".to_string(),
            },
            Self {
                question: "What networks are supported?".to_string(),
                answer: "BitVault supports:\n- Mainnet (Bitcoin)\n- Testnet (for testing)\n- Signet (Bitcoin test network)\n- Regtest (local testing)\n\nYou can switch networks in Settings.".to_string(),
            },
            Self {
                question: "How do I recover my vault?".to_string(),
                answer: "To recover your vault:\n1. Go to Recovery in the menu\n2. Select UTXOs that need recovery\n3. Create a recovery transaction\n4. Sign and broadcast the recovery transaction\n\nYou'll need your seed phrase and co-owner's keys to recover.".to_string(),
            },
            Self {
                question: "What is a hardware wallet?".to_string(),
                answer: "A hardware wallet is a physical device that stores your private keys offline. BitVault supports signing transactions with hardware wallets via QR code exchange. This adds an extra layer of security.".to_string(),
            },
        ]
    }
}
