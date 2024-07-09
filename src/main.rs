//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

fn main() {
    let our_args: Vec<String> = std::env::args().skip(1).collect();
    let (env_mod, child_cmdline) = parse_args(&our_args);

    if child_cmdline.is_empty() {
        bail("No command provided to run.");
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
        .unwrap_or(1);

    std::process::exit(status);
}

struct EnvironmentModification {
    clear_env: bool,
    unset_vars: Vec<String>,
    set_vars: Vec<(String, String)>,
}

fn bail(message: &str) -> ! {
    eprintln!("Error: {message}");
    std::process::exit(2);
}

/// Use custom rules to parse options, `NAME=VALUE` pairs, and the command to run.
///
/// This uses custom parsing because argument parsing libraries like Clap don't
/// don't readily support the specific requirements of an `env`-like utility,
/// such as arbitrary interleaving of options and `NAME=VALUE` pairs up until
/// the command to be executed.
fn parse_args(args: &[String]) -> (EnvironmentModification, Vec<String>) {
    let mut env_mod = EnvironmentModification {
        clear_env: false,
        unset_vars: Vec::new(),
        set_vars: Vec::new(),
    };
    let mut child_cmdline = Vec::new();
    let mut i = 0;

    while i < args.len() {
        // TODO: Handle '--' to explicitly mark the end of options.
        match args[i].as_str() {
            // TODO: Support combining short options (e.g., -iu VARNAME).
            "-i" | "--ignore-environment" => env_mod.clear_env = true,
            // TODO: Support long options with '=' (e.g., --unset=VARNAME).
            "-u" | "--unset" => {
                if i + 1 >= args.len() {
                    bail("-u/--unset requires an argument");
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
                child_cmdline = args[i..].to_vec();
                break;
            }
        }
        i += 1;
    }

    (env_mod, child_cmdline)
}
