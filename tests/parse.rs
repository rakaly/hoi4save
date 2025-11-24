use hoi4save::{
    file::Hoi4SliceFileKind, models::Hoi4Save, BasicTokenResolver, Encoding, Hoi4Date, Hoi4File,
    MeltOptions, PdsDate,
};
use jomini::binary::TokenResolver;
use serde::Deserialize;
use std::{error::Error, sync::LazyLock};

mod utils;

static TOKENS: LazyLock<BasicTokenResolver> = LazyLock::new(|| {
    let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
    BasicTokenResolver::from_text_lines(file_data.as_slice()).unwrap()
});

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::inflate(utils::request_file("1.10-normal-text.zip"));
    let file = Hoi4File::from_slice(&data)?;
    let save = file.parse_save(&*TOKENS)?;
    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_hoi4_text_custom_deserialization_file() -> Result<(), Box<dyn Error>> {
    let file = utils::inflate(utils::request_file("1.10-normal-text.zip"));
    let hoi4file = Hoi4File::from_slice(&file)?;
    let Hoi4SliceFileKind::Text(hoi4txt) = hoi4file.kind() else {
        panic!("expected text file kind");
    };

    #[derive(Deserialize, Debug, Clone)]
    pub struct CustomHoi4Save {
        pub date: Hoi4Date,
    }

    let save: CustomHoi4Save = hoi4txt.deserializer().deserialize()?;
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

    let data = utils::inflate(utils::request_file("1.10-normal.zip"));
    let file = Hoi4File::from_slice(&data)?;
    let save = file.parse_save(&*TOKENS)?;
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

    let data = utils::inflate(utils::request_file("1.10-ironman.zip"));
    let file = Hoi4File::from_slice(&data)?;
    let save = file.parse_save(&*TOKENS)?;
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
    let data = utils::inflate(utils::request_file("1.10-normal.zip"));

    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();
    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

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

    let data = utils::inflate(utils::request_file("1.10-ironman.zip"));
    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();
    let melted_data = utils::inflate(utils::request_file("1.10-ironman_melted.zip"));
    assert!(eq(melted_data.as_slice(), &out), "unexpected melted data");

    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

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

#[test]
fn test_ironman_roundtrip_with_nulls() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    use std::io::Cursor;

    let data = utils::inflate(utils::request_file("nulls.zip"));
    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();

    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("USA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}
