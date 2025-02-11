use hoi4save::{BasicTokenResolver, Hoi4File};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
    let file = std::fs::File::open(&args[1])?;
    let file = Hoi4File::from_file(file)?;
    let save = file.parse_save(resolver)?;
    println!("{:#?}", save);
    Ok(())
}
