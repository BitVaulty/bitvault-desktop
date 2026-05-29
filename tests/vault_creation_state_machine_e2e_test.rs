//! E2E Tests for Vault Creation State Machine
//!
//! Tests complete state machine flows for vault creation:
//! - Main device creation flow
//! - Co-owner device flow
//! - View-only mode flow
//! - Restore from backup flow
//! - State transitions, back navigation, and validation

use bitvault_app::ui::vault_creation::{DeviceRole, VaultCreationState, VaultCreationStep};

/// Helper to create a default vault creation state
fn create_default_state() -> VaultCreationState {
    VaultCreationState::default()
}

/// Advance through legal acknowledgment gates (iOS parity) before name vault.
fn advance_past_legal_gates(state: &mut VaultCreationState) {
    state.advance_to_step(VaultCreationStep::LegalTermsAcknowledgment);
    state.advance_to_step(VaultCreationStep::LegalPrivacyAcknowledgment);
    state.advance_to_step(VaultCreationStep::NameVault);
}

#[test]
fn test_vault_creation_main_device_state_flow() {
    // Test: Complete state machine flow for main device creation
    let mut state = create_default_state();
    state.device_role = DeviceRole::Main;

    // Verify initial state
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.device_role, DeviceRole::Main);

    // Expected flow for main device:
    // RoleSelection → LegalTerms → LegalPrivacy → NameVault → SetTimeDelay → MnemonicGeneration →
    // DisplaySeedPhrase → VerifySeedPhrase → SetPin → ScanCoownerKeys →
    // EmailAuth → CreateVault → DisplayExchangeData → Completed

    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::LegalTermsAcknowledgment)
    );

    // Step 1: RoleSelection → legal gates → NameVault
    advance_past_legal_gates(&mut state);
    assert_eq!(state.current_step, VaultCreationStep::NameVault);
    assert_eq!(state.step_history.len(), 3);
    assert_eq!(state.step_history[0], VaultCreationStep::RoleSelection);

    // Verify next_step_for_role() returns correct next step
    let next_step = state.next_step_for_role();
    assert_eq!(next_step, Some(VaultCreationStep::SetTimeDelay));

    // Step 2: NameVault → SetTimeDelay
    state.advance_to_step(VaultCreationStep::SetTimeDelay);
    assert_eq!(state.current_step, VaultCreationStep::SetTimeDelay);
    assert_eq!(state.step_history.len(), 4);

    // Step 3: SetTimeDelay → MnemonicGeneration
    state.advance_to_step(VaultCreationStep::MnemonicGeneration);
    assert_eq!(state.current_step, VaultCreationStep::MnemonicGeneration);

    // Step 4: MnemonicGeneration → DisplaySeedPhrase
    state.advance_to_step(VaultCreationStep::DisplaySeedPhrase);
    assert_eq!(state.current_step, VaultCreationStep::DisplaySeedPhrase);

    // Step 5: DisplaySeedPhrase → VerifySeedPhrase
    state.advance_to_step(VaultCreationStep::VerifySeedPhrase);
    assert_eq!(state.current_step, VaultCreationStep::VerifySeedPhrase);

    // Step 6: VerifySeedPhrase → SetPin
    state.advance_to_step(VaultCreationStep::SetPin);
    assert_eq!(state.current_step, VaultCreationStep::SetPin);

    // Step 7: SetPin → ScanCoownerKeys
    state.advance_to_step(VaultCreationStep::ScanCoownerKeys);
    assert_eq!(state.current_step, VaultCreationStep::ScanCoownerKeys);

    // Step 8: ScanCoownerKeys → EmailAuth
    state.advance_to_step(VaultCreationStep::EmailAuth);
    assert_eq!(state.current_step, VaultCreationStep::EmailAuth);

    // Step 9: EmailAuth → CreateVault
    state.advance_to_step(VaultCreationStep::CreateVault);
    assert_eq!(state.current_step, VaultCreationStep::CreateVault);

    // Step 10: CreateVault → DisplayExchangeData
    state.advance_to_step(VaultCreationStep::DisplayExchangeData);
    assert_eq!(state.current_step, VaultCreationStep::DisplayExchangeData);

    // Step 11: DisplayExchangeData → Completed
    state.advance_to_step(VaultCreationStep::Completed);
    assert_eq!(state.current_step, VaultCreationStep::Completed);

    // Verify complete history (13 steps including legal gates)
    assert_eq!(state.step_history.len(), 13);
}

#[test]
fn test_vault_creation_coowner_state_flow() {
    // Test: Complete state machine flow for co-owner device
    let mut state = create_default_state();
    state.device_role = DeviceRole::Coowner;

    // Verify initial state
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.device_role, DeviceRole::Coowner);

    // Expected flow for co-owner:
    // RoleSelection → LegalTerms → LegalPrivacy → NameVault → MnemonicGeneration → DisplaySeedPhrase →
    // VerifySeedPhrase → SetPin → DisplayOwnKeys → EnterExchangeData →
    // EmailAuth → CreateVault → Completed

    // Step 1: RoleSelection → legal gates → NameVault
    advance_past_legal_gates(&mut state);
    assert_eq!(state.current_step, VaultCreationStep::NameVault);

    // Verify next_step_for_role() returns correct next step (skips SetTimeDelay)
    let next_step = state.next_step_for_role();
    assert_eq!(next_step, Some(VaultCreationStep::MnemonicGeneration));

    // Step 2: NameVault → MnemonicGeneration (skips SetTimeDelay)
    state.advance_to_step(VaultCreationStep::MnemonicGeneration);
    assert_eq!(state.current_step, VaultCreationStep::MnemonicGeneration);

    // Step 3: MnemonicGeneration → DisplaySeedPhrase
    state.advance_to_step(VaultCreationStep::DisplaySeedPhrase);
    assert_eq!(state.current_step, VaultCreationStep::DisplaySeedPhrase);

    // Step 4: DisplaySeedPhrase → VerifySeedPhrase
    state.advance_to_step(VaultCreationStep::VerifySeedPhrase);
    assert_eq!(state.current_step, VaultCreationStep::VerifySeedPhrase);

    // Step 5: VerifySeedPhrase → SetPin
    state.advance_to_step(VaultCreationStep::SetPin);
    assert_eq!(state.current_step, VaultCreationStep::SetPin);

    // Step 6: SetPin → DisplayOwnKeys (co-owner specific)
    state.advance_to_step(VaultCreationStep::DisplayOwnKeys);
    assert_eq!(state.current_step, VaultCreationStep::DisplayOwnKeys);

    // Step 7: DisplayOwnKeys → EnterExchangeData
    state.advance_to_step(VaultCreationStep::EnterExchangeData);
    assert_eq!(state.current_step, VaultCreationStep::EnterExchangeData);

    // Step 8: EnterExchangeData → EmailAuth
    state.advance_to_step(VaultCreationStep::EmailAuth);
    assert_eq!(state.current_step, VaultCreationStep::EmailAuth);

    // Step 9: EmailAuth → CreateVault
    state.advance_to_step(VaultCreationStep::CreateVault);
    assert_eq!(state.current_step, VaultCreationStep::CreateVault);

    // Step 10: CreateVault → Completed (no DisplayExchangeData for co-owner)
    state.advance_to_step(VaultCreationStep::Completed);
    assert_eq!(state.current_step, VaultCreationStep::Completed);

    // Verify complete history (12 steps including legal gates)
    assert_eq!(state.step_history.len(), 12);
}

#[test]
fn test_vault_creation_view_only_state_flow() {
    // Test: View-only mode state machine
    let mut state = create_default_state();
    state.device_role = DeviceRole::ViewOnly;

    // Verify initial state
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.device_role, DeviceRole::ViewOnly);

    // Expected flow for view-only:
    // RoleSelection → LegalTerms → LegalPrivacy → NameVault → ScanDescriptorViewOnly → ViewOnlyComplete → Completed

    // Step 1: RoleSelection → legal gates → NameVault
    advance_past_legal_gates(&mut state);
    assert_eq!(state.current_step, VaultCreationStep::NameVault);

    // Verify next_step_for_role() returns correct next step
    let next_step = state.next_step_for_role();
    assert_eq!(next_step, Some(VaultCreationStep::ScanDescriptorViewOnly));

    // Step 2: NameVault → ScanDescriptorViewOnly (skips mnemonic generation)
    state.advance_to_step(VaultCreationStep::ScanDescriptorViewOnly);
    assert_eq!(
        state.current_step,
        VaultCreationStep::ScanDescriptorViewOnly
    );

    // Step 3: ScanDescriptorViewOnly → ViewOnlyComplete
    state.advance_to_step(VaultCreationStep::ViewOnlyComplete);
    assert_eq!(state.current_step, VaultCreationStep::ViewOnlyComplete);

    // Step 4: ViewOnlyComplete → Completed
    state.advance_to_step(VaultCreationStep::Completed);
    assert_eq!(state.current_step, VaultCreationStep::Completed);

    // Verify complete history (6 steps including legal gates)
    assert_eq!(state.step_history.len(), 6);

    // Verify view-only flow doesn't require mnemonic
    assert!(state.mnemonic.is_none());
}

#[test]
fn test_vault_creation_restore_state_flow() {
    // Test: Restore from backup state machine
    let mut state = create_default_state();
    state.device_role = DeviceRole::Restore;

    // Verify initial state
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.device_role, DeviceRole::Restore);

    // Expected flow for restore:
    // RoleSelection → LegalTerms → LegalPrivacy → NameVault → SelectSeedPhraseSize → EnterSeedPhrase → ScanDescriptorRestore → SetPin → Completed

    // Step 1: RoleSelection → legal gates → NameVault
    advance_past_legal_gates(&mut state);
    assert_eq!(state.current_step, VaultCreationStep::NameVault);

    // Verify next_step_for_role() returns correct next step
    let next_step = state.next_step_for_role();
    assert_eq!(next_step, Some(VaultCreationStep::SelectSeedPhraseSize));

    // Step 2: NameVault → SelectSeedPhraseSize
    state.advance_to_step(VaultCreationStep::SelectSeedPhraseSize);
    assert_eq!(state.current_step, VaultCreationStep::SelectSeedPhraseSize);

    // Step 3: SelectSeedPhraseSize → EnterSeedPhrase
    state.advance_to_step(VaultCreationStep::EnterSeedPhrase);
    assert_eq!(state.current_step, VaultCreationStep::EnterSeedPhrase);

    // Step 4: EnterSeedPhrase → ScanDescriptorRestore
    state.advance_to_step(VaultCreationStep::ScanDescriptorRestore);
    assert_eq!(state.current_step, VaultCreationStep::ScanDescriptorRestore);

    // Step 5: ScanDescriptorRestore → SetPin
    state.advance_to_step(VaultCreationStep::SetPin);
    assert_eq!(state.current_step, VaultCreationStep::SetPin);

    // Step 6: SetPin → Completed
    state.advance_to_step(VaultCreationStep::Completed);
    assert_eq!(state.current_step, VaultCreationStep::Completed);

    // Verify complete history (8 steps including legal gates)
    assert_eq!(state.step_history.len(), 8);

    // Verify restore flow includes seed phrase size selection, seed phrase entry, and descriptor scan
    assert!(state.step_history.contains(&VaultCreationStep::SelectSeedPhraseSize));
    assert!(state.step_history.contains(&VaultCreationStep::EnterSeedPhrase));
    assert!(state.step_history.contains(&VaultCreationStep::ScanDescriptorRestore));
}

#[test]
fn test_vault_creation_back_navigation() {
    // Test: Back button navigation through state machine
    let mut state = create_default_state();
    state.device_role = DeviceRole::Main;

    // Advance through several steps
    advance_past_legal_gates(&mut state);
    state.advance_to_step(VaultCreationStep::SetTimeDelay);
    state.advance_to_step(VaultCreationStep::MnemonicGeneration);

    // Verify current state
    assert_eq!(state.current_step, VaultCreationStep::MnemonicGeneration);
    assert_eq!(state.step_history.len(), 5);
    assert!(state.can_go_back_in_workflow());

    // Go back one step
    let went_back = state.go_to_previous_step();
    assert!(went_back);
    assert_eq!(state.current_step, VaultCreationStep::SetTimeDelay);
    assert_eq!(state.step_history.len(), 4); // History is popped

    // Go back again
    let went_back = state.go_to_previous_step();
    assert!(went_back);
    assert_eq!(state.current_step, VaultCreationStep::NameVault);
    assert_eq!(state.step_history.len(), 3);

    // Go back through legal gates
    let went_back = state.go_to_previous_step();
    assert!(went_back);
    assert_eq!(state.current_step, VaultCreationStep::LegalPrivacyAcknowledgment);
    assert_eq!(state.step_history.len(), 2);

    let went_back = state.go_to_previous_step();
    assert!(went_back);
    assert_eq!(state.current_step, VaultCreationStep::LegalTermsAcknowledgment);
    assert_eq!(state.step_history.len(), 1);

    // Go back to first step
    let went_back = state.go_to_previous_step();
    assert!(went_back);
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.step_history.len(), 0);
    assert!(!state.can_go_back_in_workflow());

    // Can't go back from first step
    let went_back = state.go_to_previous_step();
    assert!(!went_back);
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
}

#[test]
fn test_vault_creation_state_validation() {
    // Test: State validation at each step
    let mut state = create_default_state();
    state.device_role = DeviceRole::Main;

    // Test that vault name is required before advancing past NameVault
    // (This is validated in the UI, but we can verify the state structure supports it)
    state.advance_to_step(VaultCreationStep::NameVault);
    assert_eq!(state.vault_name, String::new()); // Initially empty

    // In the actual UI, validation would check vault_name before allowing advance
    // For this test, we verify the field exists and can be set
    state.vault_name = "Test Vault".to_string();
    assert!(!state.vault_name.is_empty());

    // Test time delay is set for main device
    state.advance_to_step(VaultCreationStep::SetTimeDelay);
    assert_eq!(state.time_delay_days, 0);
    assert_eq!(state.time_delay_hours, 24); // Default from Default impl

    // Set time delay
    state.time_delay_days = 1;
    state.time_delay_hours = 0;
    assert!(state.time_delay_days > 0 || state.time_delay_hours > 0);

    // Test seed phrase verification must pass
    state.advance_to_step(VaultCreationStep::VerifySeedPhrase);
    assert!(!state.seed_verification_state.initialized); // Initially false

    // Simulate verification by selecting all correct words
    // In the actual UI, this would happen through the verification flow
    if let Some(ref mnemonic) = state.mnemonic {
        let words: Vec<&str> = mnemonic.words().collect();
        for (idx, word) in words.iter().enumerate() {
            state
                .seed_verification_state
                .selected_words
                .insert(idx, word.to_string());
        }
        assert_eq!(state.seed_verification_state.selected_words.len(), 12);
    }

    // Test co-owner keys are scanned before vault creation
    state.advance_to_step(VaultCreationStep::ScanCoownerKeys);
    assert!(state.coowner_pubkeys.is_empty()); // Initially empty

    // In the actual UI, scanning would populate this
    state.coowner_pubkeys = "test_keys".to_string();
    assert!(!state.coowner_pubkeys.is_empty());
}

#[test]
fn test_vault_creation_state_reset() {
    // Test: State reset functionality
    let mut state = create_default_state();
    state.device_role = DeviceRole::Main;

    // Advance through several steps and set some data
    advance_past_legal_gates(&mut state);
    state.vault_name = "Test Vault".to_string();
    state.advance_to_step(VaultCreationStep::SetTimeDelay);
    state.time_delay_days = 1;
    state.advance_to_step(VaultCreationStep::MnemonicGeneration);

    // Verify state has data
    assert_eq!(state.current_step, VaultCreationStep::MnemonicGeneration);
    assert!(!state.vault_name.is_empty());
    assert!(state.time_delay_days > 0);
    assert!(!state.step_history.is_empty());

    // Test reset() - clears step history but keeps other state
    state.reset();
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    assert_eq!(state.step_history.len(), 0);
    // Note: reset() doesn't clear vault_name or time_delay_days
    // (This is by design - user might want to resume)

    // Test reset_for_new_flow() - clears all input state
    state.vault_name = "Test Vault".to_string();
    state.time_delay_days = 1;
    state.time_delay_hours = 0;
    state.mnemonic = Some(bdk::keys::bip39::Mnemonic::from_entropy(&[0u8; 16]).unwrap());
    // Simulate verification by selecting all correct words
    if let Some(ref mnemonic) = state.mnemonic {
        let words: Vec<&str> = mnemonic.words().collect();
        for (idx, word) in words.iter().enumerate() {
            state
                .seed_verification_state
                .selected_words
                .insert(idx, word.to_string());
        }
    }

    state.reset_for_new_flow();

    // Verify all input state is cleared
    assert_eq!(state.mnemonic, None);
    assert!(!state.seed_verification_state.initialized);
    assert_eq!(state.time_delay_days, 0);
    assert_eq!(state.time_delay_hours, 24); // Reset to default
    assert_eq!(state.step_history.len(), 0);
}

#[test]
fn test_vault_creation_next_step_for_role() {
    // Test: next_step_for_role() returns correct next step for each role
    let mut state = create_default_state();

    // Test main device flow
    state.device_role = DeviceRole::Main;
    state.current_step = VaultCreationStep::RoleSelection;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::LegalTermsAcknowledgment)
    );

    state.current_step = VaultCreationStep::LegalTermsAcknowledgment;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::LegalPrivacyAcknowledgment)
    );

    state.current_step = VaultCreationStep::LegalPrivacyAcknowledgment;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::NameVault)
    );

    state.current_step = VaultCreationStep::NameVault;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::SetTimeDelay)
    );

    state.current_step = VaultCreationStep::SetTimeDelay;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::MnemonicGeneration)
    );

    // Test co-owner flow (skips SetTimeDelay)
    state.device_role = DeviceRole::Coowner;
    state.current_step = VaultCreationStep::NameVault;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::MnemonicGeneration)
    );

    // Test view-only flow
    state.device_role = DeviceRole::ViewOnly;
    state.current_step = VaultCreationStep::NameVault;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::ScanDescriptorViewOnly)
    );

    // Test restore flow (NameVault → SelectSeedPhraseSize → EnterSeedPhrase → ...)
    state.device_role = DeviceRole::Restore;
    state.current_step = VaultCreationStep::NameVault;
    assert_eq!(
        state.next_step_for_role(),
        Some(VaultCreationStep::SelectSeedPhraseSize)
    );
}

#[test]
fn test_vault_creation_step_just_changed() {
    // Test: step_just_changed() detects step transitions
    // This method is used for auto-focus in the UI - it returns true only once after a step change
    let mut state = create_default_state();

    // Initially, previous_step is None, current_step is RoleSelection
    // step_just_changed compares previous_step with the parameter
    // Since previous_step is None, checking RoleSelection should return true (None != Some(RoleSelection))
    // But actually, None.as_ref() is None, and Some(&RoleSelection) != None, so it returns true
    // However, the method is meant to be called with the current step, so let's test it properly

    // After advance_to_step, previous_step is set to the old current_step
    advance_past_legal_gates(&mut state);
    // Now: current_step = NameVault, previous_step updated through legal flow

    // step_just_changed(NameVault) compares previous_step with NameVault
    let changed = state.step_just_changed(VaultCreationStep::NameVault);
    assert!(
        changed,
        "step_just_changed should return true when step has changed"
    );

    // After calling step_just_changed, it updates previous_step to the parameter value
    // So previous_step is now Some(NameVault)
    // Checking NameVault again should return false (Some(NameVault) == Some(NameVault))
    assert!(
        !state.step_just_changed(VaultCreationStep::NameVault),
        "step_just_changed should return false after previous_step is updated"
    );

    // If we check a different step, it should return true again
    assert!(
        state.step_just_changed(VaultCreationStep::SetTimeDelay),
        "step_just_changed should return true for a different step"
    );
}
