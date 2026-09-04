use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

const USAGE: &str = "usage: linefold [width] [--indent N]\n\n\
Reads text from stdin and writes it wrapped to `width` columns\n\
(default 80) on stdout. --indent N indents every wrapped line by N\n\
spaces, packing words into width - N columns so the indent doesn't\n\
push lines past `width`.";

/// What `main` should do once arguments have been parsed, kept separate
/// from parsing itself so the parsing logic can be tested without going
/// through stdin or process exit codes.
#[derive(Debug, PartialEq)]
enum Action {
    Run { width: usize, indent: usize },
    Help,
    Error(String),
}

fn parse_args(args: &[String]) -> Action {
    let mut width = 80usize;
    let mut indent = 0usize;
    let mut width_set = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => return Action::Help,
            "--indent" => {
                i += 1;
                let value = match args.get(i) {
                    Some(v) => v,
                    None => return Action::Error("--indent requires a value".to_string()),
                };
                match value.parse::<usize>() {
                    Ok(n) => indent = n,
                    Err(_) => {
                        return Action::Error(format!("'{value}' is not a valid indent"))
                    }
                }
            }
            arg if !width_set => match arg.parse::<usize>() {
                Ok(w) => {
                    width = w;
                    width_set = true;
                }
                Err(_) => return Action::Error(format!("'{arg}' is not a valid width")),
            },
            arg => return Action::Error(format!("unexpected argument '{arg}'")),
        }
        i += 1;
    }

    Action::Run { width, indent }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let (width, indent) = match parse_args(&args) {
        Action::Help => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Action::Error(msg) => {
            eprintln!("linefold: {msg}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
        Action::Run { width, indent } => (width, indent),
    };

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("linefold: failed to read stdin: {err}");
        return ExitCode::FAILURE;
    }

    println!("{}", linefold::wrap_indented(&input, width, indent));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    struct Case {
        name: &'static str,
        args: &'static [&'static str],
        expected: Action,
    }

    #[test]
    fn parse_args_table() {
        let cases = vec![
            Case {
                name: "no arguments uses the defaults",
                args: &[],
                expected: Action::Run { width: 80, indent: 0 },
            },
            Case {
                name: "a bare number sets the width",
                args: &["60"],
                expected: Action::Run { width: 60, indent: 0 },
            },
            Case {
                name: "--indent alone leaves width at the default",
                args: &["--indent", "4"],
                expected: Action::Run { width: 80, indent: 4 },
            },
            Case {
                name: "width and --indent combine",
                args: &["60", "--indent", "4"],
                expected: Action::Run { width: 60, indent: 4 },
            },
            Case {
                name: "--indent before the width still works",
                args: &["--indent", "4", "60"],
                expected: Action::Run { width: 60, indent: 4 },
            },
            Case {
                name: "-h requests help",
                args: &["-h"],
                expected: Action::Help,
            },
            Case {
                name: "--help requests help",
                args: &["--help"],
                expected: Action::Help,
            },
            Case {
                name: "--indent with no value is an error",
                args: &["--indent"],
                expected: Action::Error("--indent requires a value".to_string()),
            },
            Case {
                name: "--indent with a non-numeric value is an error",
                args: &["--indent", "many"],
                expected: Action::Error("'many' is not a valid indent".to_string()),
            },
            Case {
                name: "a non-numeric width is an error",
                args: &["wide"],
                expected: Action::Error("'wide' is not a valid width".to_string()),
            },
            Case {
                name: "a second bare number is an unexpected argument",
                args: &["60", "70"],
                expected: Action::Error("unexpected argument '70'".to_string()),
            },
        ];

        let mut failures = Vec::new();
        for case in &cases {
            let got = parse_args(&args(case.args));
            if got != case.expected {
                failures.push(format!(
                    "case '{}' failed:\n  args:     {:?}\n  expected: {:?}\n  got:      {:?}",
                    case.name, case.args, case.expected, got
                ));
            }
        }

        assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
    }
}
