use mactest::MacTest;

#[test]
fn test_mactest() {
    let mut instance = MacTest::new();
    assert!(!instance.finished());
    assert!(!instance.finished());
    assert!(!instance.finished());
}
