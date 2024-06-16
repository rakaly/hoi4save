use hoi4save::{models::Hoi4Save, Encoding, EnvTokens, Hoi4File, PdsDate};
use std::error::Error;

mod utils;

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal-text.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&EnvTokens).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_normal_bin() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&EnvTokens).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_hoi4_ironman() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-ironman.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&EnvTokens).deserialize()?;
    assert_eq!(file.encoding(), Encoding::Binary);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_normal_roundtrip() -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;
    let data = utils::request("1.10-normal.zip");

    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    file.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&mut out, &EnvTokens)?;

    let out = out.into_inner();
    let file = Hoi4File::from_slice(&out)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&EnvTokens).deserialize()?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_ironman_roundtrip() -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;

    let data = utils::request("1.10-ironman.zip");
    let file = Hoi4File::from_slice(&data)?;
    let mut out = Cursor::new(Vec::new());
    file.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&mut out, &EnvTokens)?;

    let out = out.into_inner();
    let melted_data = utils::request("1.10-ironman_melted.zip");
    assert!(eq(melted_data.as_slice(), &out), "unexpected melted data");

    let file = Hoi4File::from_slice(&out)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer(&EnvTokens).deserialize()?;

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
