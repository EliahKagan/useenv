//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

fn main() {
    let our_args: Vec<String> = std::env::args().skip(1).collect();
    let (env_mod, child_cmdline) = parse_args(&our_args);

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
        .expect("Subprocess terminated abnormally");

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
///
/// This does not handle `--`, combined short options (e.g. `-iu`), or long
/// options with `=` (i.e. no `--unset=NAME`; use `--unset NAME` or `-u NAME`).
fn parse_args(args: &[String]) -> (EnvironmentModification, Vec<String>) {
    let mut env_mod = EnvironmentModification::default();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--ignore-environment" => env_mod.clear_env = true,
            "-u" | "--unset" => {
                if i + 1 >= args.len() {
                    panic!("-u/--unset requires an argument");
                }
                env_mod.unset_vars.push(args[i + 1].clone());
                i += 1;
            }
            arg if arg.contains('=') => {
                if let Some((name, value)) = arg.split_once('=') {
                    env_mod.set_vars.push((name.to_string(), value.to_string()));
                }
            }
            _ => {
                let child_cmdline = args[i..].to_vec();
                return (env_mod, child_cmdline);
            }
        }
        i += 1;
    }

    panic!("No child command provided to run");
}
