use std::io::Error;

use mactest::Scandir;

#[test]
fn test_mactest() -> Result<(), Error> {
    let mut instance = Scandir::new(".", None)?;
    assert!(!instance.finished());
    assert!(!instance.finished());
    assert!(!instance.finished());
    Ok(())
}
