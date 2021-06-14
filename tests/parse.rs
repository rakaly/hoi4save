use hoi4save::{Encoding, Hoi4Extractor, PdsDate};
use std::error::Error;

mod utils;

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal-text.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(save.date.game_fmt().to_string(), String::from("1936.1.1.12"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_normal_bin() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(save.date.game_fmt().to_string(), String::from("1936.1.1.12"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_ironman() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-ironman.zip");
    let (save, encoding) = Hoi4Extractor::builder().extract_save(&data)?;
    assert_eq!(encoding, Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(save.date.game_fmt().to_string(), String::from("1936.1.1.12"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_normal_roundtrip() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal.zip");
    let (melted, _tokens) = hoi4save::Melter::new()
        .with_on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&data[..])
        .unwrap();

    let (save, encoding) = Hoi4Extractor::builder().extract_save(&melted)?;
    assert_eq!(encoding, Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(save.date.game_fmt().to_string(), String::from("1936.1.1.12"));
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_ironman_roundtrip() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-ironman.zip");
    let (melted, _tokens) = hoi4save::Melter::new()
        .with_on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&data[..])
        .unwrap();

    let (save, encoding) = Hoi4Extractor::builder().extract_save(&melted)?;
    assert_eq!(encoding, Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(save.date.game_fmt().to_string(), String::from("1936.1.1.12"));
    Ok(())
}
