use hoi4save::{
    file::{Hoi4FsFileKind, Hoi4ParsedText},
    BasicTokenResolver, Hoi4File,
};
use std::{env, io::Read};

fn json_to_stdout(file: &Hoi4ParsedText) {
    let _ = file.reader().json().to_writer(std::io::stdout());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = std::fs::File::open(&args[1]).unwrap();
    let mut file = Hoi4File::from_file(file)?;
    match file.kind_mut() {
        Hoi4FsFileKind::Text(x) => {
            let mut buf = Vec::new();
            x.read_to_end(&mut buf)?;
            let text = Hoi4ParsedText::from_raw(&buf)?;
            json_to_stdout(&text);
        }
        Hoi4FsFileKind::Binary(x) => {
            let file_data = std::fs::read("assets/hoi4.txt").unwrap_or_default();
            let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
            let melt_options = hoi4save::MeltOptions::new().verbatim(true);
            let mut buf = Vec::new();
            x.melt(melt_options, resolver, &mut buf)?;
            let text = Hoi4ParsedText::from_slice(&buf)?;
            json_to_stdout(&text);
        }
    }
    Ok(())
}
