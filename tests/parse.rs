use hoi4save::{models::Hoi4Save, BasicTokenResolver, Encoding, Hoi4File, PdsDate};
use jomini::binary::TokenResolver;
use std::{error::Error, sync::LazyLock};

mod utils;

static TOKENS: LazyLock<BasicTokenResolver> = LazyLock::new(|| {
    let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
    BasicTokenResolver::from_text_lines(file_data.as_slice()).unwrap()
});

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal-text.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&*TOKENS).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_hoi4_normal_bin() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    let data = utils::request("1.10-normal.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&*TOKENS).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_hoi4_ironman() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    let data = utils::request("1.10-ironman.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&*TOKENS).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_normal_roundtrip() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    use std::io::Cursor;
    let data = utils::request("1.10-normal.zip");

    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    file.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&mut out, &*TOKENS)?;

    let out = out.into_inner();
    let file = Hoi4File::from_slice(&out)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&*TOKENS).deserialize()?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_ironman_roundtrip() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    use std::io::Cursor;

    let data = utils::request("1.10-ironman.zip");
    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    file.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&mut out, &*TOKENS)?;

    let out = out.into_inner();
    let melted_data = utils::request("1.10-ironman_melted.zip");
    assert!(eq(melted_data.as_slice(), &out), "unexpected melted data");

    let file = Hoi4File::from_slice(&out)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&*TOKENS).deserialize()?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

fn eq(a: &[u8], b: &[u8]) -> bool {
    for (ai, bi) in a.iter().zip(b.iter()) {
        if ai != bi {
            return false;
        }
    }

    a.len() == b.len()
}
