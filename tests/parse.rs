use std::error::Error;
use hoi4save::{Encoding, Hoi4Extractor};

mod utils;

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal-text.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_normal_bin() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_ironman() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-ironman.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    Ok(())
}
