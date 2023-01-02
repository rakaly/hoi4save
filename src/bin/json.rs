use hoi4save::{
    file::{Hoi4ParsedFile, Hoi4ParsedFileKind, Hoi4Text},
    EnvTokens, Hoi4File,
};
use std::env;

fn json_to_stdout(file: &Hoi4Text) {
    let _ = file.reader().json().to_writer(std::io::stdout());
}

fn parsed_file_to_json(file: &Hoi4ParsedFile) -> Result<(), Box<dyn std::error::Error>> {
    // if the save is binary, melt it, as the JSON API only works with text
    match file.kind() {
        Hoi4ParsedFileKind::Text(text) => json_to_stdout(text),
        Hoi4ParsedFileKind::Binary(binary) => {
            let melted = binary.melter().verbatim(true).melt(&EnvTokens)?;
            json_to_stdout(&Hoi4Text::from_slice(melted.data())?);
        }
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();

    let file = Hoi4File::from_slice(&data)?;
    let file = file.parse()?;
    parsed_file_to_json(&file)?;

    Ok(())
}
