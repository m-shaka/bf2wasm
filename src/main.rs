extern crate clap;

use clap::{App, Arg};

use bf2wasm::compile;
use std::io::Write;
use std::{fs::File, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("bf2wasm")
        .arg(
            Arg::with_name("input")
                .help("input brainfuck file path")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .help("output wasm file path")
                .short("o")
                .takes_value(true),
        );
    let matches = app.get_matches();
    let input_path = matches.value_of("input").expect("input is required");
    let output_path = match matches.value_of("output") {
        None => {
            let input_path = Path::new(input_path);
            input_path
                .file_stem()
                .and_then(|s| s.to_str().map(|s_| format!("{}.wasm", s_)))
                .unwrap()
        }
        Some(p) => p.to_string(),
    };
    let code = compile(input_path);
    let mut out = File::create(output_path)?;
    out.write_all(&code)?;
    Ok(())
}
