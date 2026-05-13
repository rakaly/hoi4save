use highway::HighwayHash;
use hoi4save::{
    file::Hoi4FsFileKind, models::Hoi4Save, BasicTokenResolver, Encoding, Hoi4Date, Hoi4File,
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
    let file = utils::request_file("1.10-normal-text.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let save = file.parse_save(&*TOKENS)?;
    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player.as_deref(), Some("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_hoi4_text_custom_deserialization_file() -> Result<(), Box<dyn Error>> {
    let file = utils::request_file("1.10-normal-text.hoi4");
    let hoi4file = Hoi4File::from_file(file)?;
    let Hoi4FsFileKind::Text(hoi4txt) = hoi4file.kind() else {
        panic!("expected text file kind");
    };

    #[derive(Deserialize, Debug, Clone)]
    pub struct CustomHoi4Save {
        pub date: Hoi4Date,
    }

    let save: CustomHoi4Save = hoi4txt.as_ref().deserializer().deserialize()?;
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

    let file = utils::request_file("1.10-normal.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let save = file.parse_save(&*TOKENS)?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player.as_deref(), Some("FRA"));
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

    let file = utils::request_file("1.10-ironman.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let save = file.parse_save(&*TOKENS)?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player.as_deref(), Some("FRA"));
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
    let file = utils::request_file("1.10-normal.hoi4");

    let mut file = Hoi4File::from_file(file)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();
    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player.as_deref(), Some("FRA"));
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

    let file = utils::request_file("1.10-ironman.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();
    let hash = highway::HighwayHasher::default().hash256(&out);
    let checksum = format!(
        "{:016x}{:016x}{:016x}{:016x}",
        hash[0], hash[1], hash[2], hash[3]
    );
    assert_eq!(
        &checksum,
        "6e8f589e8d181c5205d051834617abbb7b90b4d16b3062d0ac0b85474fe41aa1"
    );

    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player.as_deref(), Some("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[test]
fn test_comp_bin_melt_checksum() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    use std::io::Cursor;

    let file = utils::request_file("comp_bin.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();
    let hash = highway::HighwayHasher::default().hash256(&out);
    let checksum = format!(
        "{:016x}{:016x}{:016x}{:016x}",
        hash[0], hash[1], hash[2], hash[3]
    );
    assert_eq!(
        &checksum,
        "8fbd2292046f67eb76aa50dcddc79aa75d8bee25ef7088c1aaff3d0882a48838"
    );
    Ok(())
}

#[test]
fn test_ironman_roundtrip_with_nulls() -> Result<(), Box<dyn Error>> {
    if TOKENS.is_empty() {
        return Ok(());
    }

    use std::io::Cursor;

    let file = utils::request_file("1.17-new-ironman-format.hoi4");
    let mut file = Hoi4File::from_file(file)?;
    let mut out = Cursor::new(Vec::new());
    let options = MeltOptions::new().on_failed_resolve(hoi4save::FailedResolveStrategy::Error);
    file.melt(options, &*TOKENS, &mut out)?;

    let out = out.into_inner();

    let file = Hoi4File::from_slice(&out)?;
    let save: Hoi4Save = file.parse_save(&*TOKENS)?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player.as_deref(), Some("USA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}
