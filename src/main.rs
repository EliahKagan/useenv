//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

use std::process;

use clap::{crate_version, Arg};

fn main() {
    let matches = clap::Command::new("useenv")
        .version(crate_version!())
        .about("Run a program with a modified environment")
        .arg(
            Arg::new("args")
                .num_args(0..)
                .allow_hyphen_values(true)
                .trailing_var_arg(true),
        )
        .get_matches();

    let args: Vec<String> = matches
        .get_many::<String>("args")
        .unwrap_or_default()
        .cloned()
        .collect();

    let (clear_env, unset_vars, set_vars, child_cmdline) = parse_args(&args);

    if child_cmdline.is_empty() {
        eprintln!("Error: No command provided to run.");
        process::exit(2);
    }

    let mut child = process::Command::new(&child_cmdline[0]);
    child.args(&child_cmdline[1..]);

    if clear_env {
        child.env_clear();
    }
    for var in unset_vars {
        child.env_remove(var);
    }
    child.envs(set_vars);

    let status = child
        .spawn()
        .expect("Failed to execute command")
        .wait()
        .expect("Failed to wait on child process")
        .code()
        .unwrap_or(1);

    process::exit(status);
}

/// Use custom rules to parse options, `NAME=VALUE` pairs, and the command to run.
fn parse_args(args: &[String]) -> (bool, Vec<String>, Vec<(String, String)>, Vec<String>) {
    let mut clear_env = false;
    let mut unset_vars = Vec::new();
    let mut set_vars = Vec::new();
    let mut child_cmdline = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--ignore-environment" => clear_env = true,
            "-u" | "--unset" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: -u/--unset requires an argument");
                    process::exit(2);
                }
                unset_vars.push(args[i + 1].clone());
                i += 1;
            }
            arg if arg.contains('=') => {
                if let Some((name, value)) = arg.split_once('=') {
                    set_vars.push((name.to_string(), value.to_string()));
                }
            }
            _ => {
                child_cmdline = args[i..].to_vec();
                break;
            }
        }
        i += 1;
    }

    (clear_env, unset_vars, set_vars, child_cmdline)
}
