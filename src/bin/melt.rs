use hoi4save::{BasicTokenResolver, FailedResolveStrategy, Hoi4File};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let data = std::fs::read(&args[1])?;
    let file = Hoi4File::from_slice(&data)?;
    let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
    let stdout = std::io::stdout();
    file.melter()
        .on_failed_resolve(FailedResolveStrategy::Error)
        .melt(stdout.lock(), &resolver)?;

    Ok(())
}
