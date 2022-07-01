use hoi4save::{Encoding, PdsDate, Hoi4File, models::Hoi4Save, EnvTokens};
use std::error::Error;

mod utils;

#[test]
fn test_hoi4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("1.10-normal-text.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer().build(&EnvTokens)?;
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
    let save: Hoi4Save = parsed_file.deserializer().build(&EnvTokens)?;
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
    let save: Hoi4Save = parsed_file.deserializer().build(&EnvTokens)?;
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
    let data = utils::request("1.10-normal.zip");

    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let binary = parsed_file.as_binary().unwrap();
    let out = binary.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&EnvTokens)?;

    let file = Hoi4File::from_slice(out.data())?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer().build(&EnvTokens)?;

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
    let data = utils::request("1.10-ironman.zip");
    let file = Hoi4File::from_slice(&data)?;
    let parsed_file = file.parse()?;
    let binary = parsed_file.as_binary().unwrap();
    let out = binary.melter()
        .on_failed_resolve(hoi4save::FailedResolveStrategy::Error)
        .melt(&EnvTokens)?;

    let file = Hoi4File::from_slice(out.data())?;
    let parsed_file = file.parse()?;
    let save: Hoi4Save = parsed_file.deserializer().build(&EnvTokens)?;

    assert_eq!(file.encoding(), Encoding::Plaintext);
    assert_eq!(save.player, String::from("FRA"));
    assert_eq!(
        save.date.game_fmt().to_string(),
        String::from("1936.1.1.12")
    );
    Ok(())
}
