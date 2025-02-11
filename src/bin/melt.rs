use hoi4save::{BasicTokenResolver, FailedResolveStrategy, Hoi4File, MeltOptions};
use std::{env, io::BufWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = std::fs::File::open(&args[1])?;
    let mut file = Hoi4File::from_file(file)?;
    let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
    let stdout = std::io::stdout();
    let options = MeltOptions::new().on_failed_resolve(FailedResolveStrategy::Error);
    let mut buffer = BufWriter::new(stdout.lock());
    file.melt(options, &resolver, &mut buffer)?;
    Ok(())
}
