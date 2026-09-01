use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

const USAGE: &str = "usage: linefold [width] [--indent N]\n\n\
Reads text from stdin and writes it wrapped to `width` columns\n\
(default 80) on stdout. --indent N indents every wrapped line by N\n\
spaces, packing words into width - N columns so the indent doesn't\n\
push lines past `width`.";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut width = 80usize;
    let mut indent = 0usize;
    let mut width_set = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--indent" => {
                i += 1;
                let value = match args.get(i) {
                    Some(v) => v,
                    None => {
                        eprintln!("linefold: --indent requires a value\n\n{USAGE}");
                        return ExitCode::FAILURE;
                    }
                };
                match value.parse::<usize>() {
                    Ok(n) => indent = n,
                    Err(_) => {
                        eprintln!("linefold: '{value}' is not a valid indent\n\n{USAGE}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            arg if !width_set => match arg.parse::<usize>() {
                Ok(w) => {
                    width = w;
                    width_set = true;
                }
                Err(_) => {
                    eprintln!("linefold: '{arg}' is not a valid width\n\n{USAGE}");
                    return ExitCode::FAILURE;
                }
            },
            arg => {
                eprintln!("linefold: unexpected argument '{arg}'\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
        }
        i += 1;
    }

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("linefold: failed to read stdin: {err}");
        return ExitCode::FAILURE;
    }

    println!("{}", linefold::wrap_indented(&input, width, indent));
    ExitCode::SUCCESS
}
