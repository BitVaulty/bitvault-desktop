use bitcoin::Amount;
use bitvault_common::bitcoin_utils;
mod test_utils;
use test_utils::test_with_logging;

#[test]
fn test_bitcoin_amount_creation() {
    let _ = test_with_logging("test_bitcoin_amount_creation", || {
        eprintln!("Starting test_bitcoin_amount_creation");

        // Create from satoshis
        eprintln!("Testing Amount::from_sat(123456789)");
        let amount1 = Amount::from_sat(123456789);
        eprintln!("Result: {} satoshis", amount1.to_sat());
        assert_eq!(amount1.to_sat(), 123456789);

        // Create from BTC
        eprintln!("Testing Amount::from_btc(1.23456789)");
        let amount2_result = Amount::from_btc(1.23456789);
        match &amount2_result {
            Ok(amount) => eprintln!("Successfully created amount: {} satoshis", amount.to_sat()),
            Err(e) => eprintln!("ERROR creating amount: {:?}", e),
        }

        let amount2 = amount2_result.expect("Failed to create amount from BTC");
        eprintln!("Result: {} satoshis", amount2.to_sat());
        assert_eq!(amount2.to_sat(), 123456789);

        // Verify equivalence
        eprintln!("Verifying equivalence");
        assert_eq!(amount1, amount2);

        eprintln!("Completed test_bitcoin_amount_creation");
    });
}

#[test]
fn test_bitcoin_amount_formatting() {
    let _ = test_with_logging("test_bitcoin_amount_formatting", || {
        eprintln!("Starting test_bitcoin_amount_formatting");

        let amount = Amount::from_sat(123456789);
        eprintln!("Testing with amount: {} satoshis", amount.to_sat());

        // Format as BTC
        eprintln!("Formatting as BTC");
        let btc_format = bitcoin_utils::format_bitcoin_amount(amount, true);
        eprintln!("Result: {}", btc_format);
        assert_eq!(btc_format, "1.23456789 BTC");

        // Format as satoshis
        eprintln!("Formatting as satoshis");
        let sat_format = bitcoin_utils::format_bitcoin_amount(amount, false);
        eprintln!("Result: {}", sat_format);
        assert_eq!(sat_format, "123456789 sats");

        eprintln!("Completed test_bitcoin_amount_formatting");
    });
}
