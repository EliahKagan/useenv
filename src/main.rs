//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

fn main() {
    let (env_mod, child_cmdline) = parse_our_args(std::env::args().skip(1));

    if child_cmdline.is_empty() {
        panic!("No command provided to run.");
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

/// Use custom rules to parse options, `NAME=VALUE` pairs, and the command to run.
///
/// This uses custom parsing because I did not find an elegant way with Clap to
/// allow options and `NAME=VALUE` pairs to be interleaved arbitrarily up until
/// the command to be executed.
fn parse_our_args<I>(args: I) -> (EnvironmentModification, Vec<String>)
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
            // TODO: Support combining short options (e.g., -iu VARNAME).
            "-i" | "--ignore-environment" => env_mod.clear_env = true,
            // TODO: Support long options with '=' (e.g., --unset=VARNAME).
            "-u" | "--unset" => {
                let next_arg = args_iter.next()
                    .expect("-u/--unset requires an argument");
                env_mod.unset_vars.push(next_arg.as_ref().to_string());
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

    (env_mod, child_cmdline)
}
