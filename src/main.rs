//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

use std::error::Error;
use std::fmt;

/// Custom error type for argument parsing errors.
#[derive(Debug)]
enum ParseError {
    MissingArgument(String),
    UnknownOption(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingArgument(opt) => write!(f, "{} requires an argument", opt),
            ParseError::UnknownOption(opt) => write!(f, "Unknown option: {}", opt),
        }
    }
}

impl Error for ParseError {}

fn main() -> Result<(), Box<dyn Error>> {
    let (env_mod, child_cmdline) = parse_our_args(std::env::args().skip(1))?;

    if child_cmdline.is_empty() {
        return Err("No command provided to run.".into());
    }

    let mut child = std::process::Command::new(&child_cmdline[0]);
    child.args(&child_cmdline[1..]);

    if env_mod.clear_env {
        child.env_clear();
    }
    for var in env_mod.unset_vars {
        child.env_remove(var);
    }
    child.envs(env_mod.set_vars);

    let status = child
        .spawn()
        .expect("Failed to execute command")
        .wait()
        .expect("Failed to wait on child process")
        .code()
        .unwrap_or(1); // TODO: Maybe a higher value would be better here.

    std::process::exit(status);
}

#[derive(Default)]
struct EnvironmentModification {
    clear_env: bool,
    unset_vars: Vec<String>,
    set_vars: Vec<(String, String)>,
}

/// Use custom rules to parse our program's options, `NAME=VALUE` pairs,
/// and extract the command to run.
///
/// This uses custom parsing because I did not find an elegant way with Clap to
/// allow options and `NAME=VALUE` pairs to be interleaved arbitrarily up until
/// the command to be executed.
///
/// # Arguments
///
/// * `args` - An iterator over the arguments passed to the program.
///
/// # Returns
///
/// A tuple containing:
/// - `EnvironmentModification`: The modifications to the environment.
/// - `Vec<String>`: The command and its arguments to be executed.
///
/// # Errors
///
/// Returns a `ParseError` if an unknown option is encountered or if an option
/// that requires an argument is missing one.
fn parse_our_args<I>(args: I) -> Result<(EnvironmentModification, Vec<String>), ParseError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut env_mod = EnvironmentModification::default();
    let mut child_cmdline = Vec::new();
    let mut args_iter = args.into_iter().peekable();

    while let Some(arg) = args_iter.next() {
        // TODO: Handle '--' to explicitly mark the end of options.
        match arg.as_ref() {
            arg if arg.starts_with("--") => {
                let (option, value) = arg.split_once('=').unwrap_or((arg, ""));
                match option {
                    "--ignore-environment" => env_mod.clear_env = true,
                    "--unset" => {
                        let var = if !value.is_empty() {
                            value.to_string()
                        } else {
                            args_iter.next()
                                .ok_or_else(|| ParseError::MissingArgument("--unset".to_string()))?
                                .as_ref().to_string()
                        };
                        env_mod.unset_vars.push(var);
                    }
                    _ => return Err(ParseError::UnknownOption(option.to_string())),
                }
            }
            arg if arg.starts_with('-') => {
                let mut chars = arg.chars().skip(1).peekable();
                while let Some(option_char) = chars.next() {
                    match option_char {
                        'i' => env_mod.clear_env = true,
                        'u' => {
                            let var = if chars.peek().is_some() {
                                chars.collect()
                            } else {
                                args_iter.next()
                                    .ok_or_else(|| ParseError::MissingArgument("-u".to_string()))?
                                    .as_ref().to_string()
                            };
                            env_mod.unset_vars.push(var);
                            break;
                        }
                        _ => return Err(ParseError::UnknownOption(format!("-{}", option_char))),
                    }
                }
            }
            arg if arg.contains('=') => {
                if let Some((name, value)) = arg.split_once('=') {
                    env_mod.set_vars.push((name.to_string(), value.to_string()));
                }
            }
            _ => {
                child_cmdline = std::iter::once(arg.as_ref().to_string())
                    .chain(args_iter.map(|s| s.as_ref().to_string()))
                    .collect();
                break;
            }
        }
    }

    Ok((env_mod, child_cmdline))
}
