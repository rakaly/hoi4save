use hoi4save::{EnvTokens, FailedResolveStrategy, Hoi4File};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let data = std::fs::read(&args[1])?;
    let file = Hoi4File::from_slice(&data)?;
    let stdout = std::io::stdout();
    file.melter()
        .on_failed_resolve(FailedResolveStrategy::Error)
        .melt(stdout.lock(), &EnvTokens)?;

    Ok(())
}
