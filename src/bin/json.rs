use hoi4save::{file::Hoi4Text, EnvTokens, Hoi4File};
use std::{env, io::Cursor};

fn json_to_stdout(file: &Hoi4Text) {
    let _ = file.reader().json().to_writer(std::io::stdout());
}

fn parsed_file_to_json(file: &Hoi4File) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = Cursor::new(Vec::new());
    file.melter().verbatim(true).melt(&mut out, &EnvTokens)?;
    json_to_stdout(&Hoi4Text::from_slice(out.get_ref())?);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();

    let file = Hoi4File::from_slice(&data)?;
    parsed_file_to_json(&file)?;

    Ok(())
}
