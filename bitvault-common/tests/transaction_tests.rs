use bitvault_common::bitcoin_utils;
mod test_utils;
use test_utils::test_with_logging;

#[test]
fn test_txid_validation() {
    let _ = test_with_logging("test_txid_validation", || {
        eprintln!("Starting test_txid_validation");

        // Valid txid
        let valid_txid = "3a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";
        eprintln!("Testing valid txid: {}", valid_txid);
        let res1 = bitcoin_utils::is_valid_txid(valid_txid);
        eprintln!("Result: {}", res1);
        assert!(res1);

        // Invalid txids
        eprintln!("Testing invalid txid: 'invalid'");
        let res2 = bitcoin_utils::is_valid_txid("invalid");
        eprintln!("Result: {}", res2);
        assert!(!res2);

        eprintln!("Testing empty txid");
        let res3 = bitcoin_utils::is_valid_txid("");
        eprintln!("Result: {}", res3);
        assert!(!res3);

        // Invalid characters in txid
        let invalid_chars = "3a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1Z";
        eprintln!("Testing txid with invalid chars: {}", invalid_chars);
        let res4 = bitcoin_utils::is_valid_txid(invalid_chars);
        eprintln!("Result: {}", res4);
        assert!(!res4);

        eprintln!("Completed test_txid_validation");
    });
}
