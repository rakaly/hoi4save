//! Utility to format HOI4 text files from stdin to stdout.
//!
//! Useful to compare a game generated debug file and a melted file and identify
//! differences.

use std::{
    error,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let stdin = io::stdin().lock();
    let mut buf_stdin = BufReader::new(stdin);

    let mut first_line = String::new();
    buf_stdin.read_line(&mut first_line)?;
    if first_line.trim_end() != "HOI4txt" {
        return Err(format!("expected HOI4txt header, got: {:?}", first_line.trim_end()).into());
    }

    let mut reader = jomini::text::TokenReader::new(buf_stdin);

    let stdout = io::stdout().lock();
    let mut writer = BufWriter::new(stdout);
    writer.write_all(b"HOI4txt\n")?;
    let writer = writer;
    let mut writer = jomini::TextWriterBuilder::new().from_writer(writer);

    while let Some(token) = reader.next()? {
        match token {
            jomini::text::Token::Open => {
                writer.write_start()?;
            }
            jomini::text::Token::Close => writer.write_end()?,
            jomini::text::Token::Operator(op) => writer.write_operator(op)?,
            jomini::text::Token::Unquoted(x) => writer.write_unquoted(x.as_bytes())?,
            jomini::text::Token::Quoted(x) => writer.write_quoted(x.as_bytes())?,
        }
    }

    Ok(())
}
