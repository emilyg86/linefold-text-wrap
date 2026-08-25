use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

const USAGE: &str = "usage: linefold [width]\n\nReads text from stdin and writes it wrapped to `width` columns\n(default 80) on stdout.";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    let width = match args.next() {
        None => 80,
        Some(arg) if arg == "-h" || arg == "--help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Some(arg) => match arg.parse::<usize>() {
            Ok(w) => w,
            Err(_) => {
                eprintln!("linefold: '{arg}' is not a valid width\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
        },
    };

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("linefold: failed to read stdin: {err}");
        return ExitCode::FAILURE;
    }

    println!("{}", linefold::wrap(&input, width));
    ExitCode::SUCCESS
}
