#[test]
fn test_simple_output() {
    println!("This is a simple test output");
    println!("If you can see this, output capture is working");
    eprintln!("This is output to stderr");
}

#[test]
fn test_always_passes() {
    assert!(true);
}

#[test]
#[should_panic]
fn test_intentional_panic() {
    panic!("This is an intentional panic for testing");
} 