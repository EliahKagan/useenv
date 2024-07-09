//! Runs a program with a modified environment.
//!
//! This is inspired by the Unix `env` command but not equivalent to it. This is
//! due both to non-portable aspects related to its typical implementation in
//! terms of `exec`, and to the currently very rough condition this code is in.

use std::process;

use clap::{crate_version, Arg, ArgAction, ArgMatches};

fn main() {
    let matches = clap::Command::new("useenv")
        .version(crate_version!())
        .about("Run a program with a modified environment")
        .arg(
            Arg::new("ignore-environment")
                .short('i')
                .long("ignore-environment")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("unset")
                .short('u')
                .long("unset")
                .action(ArgAction::Append)
                .num_args(1),
        )
        .arg(Arg::new("args").num_args(0..).allow_hyphen_values(true))
        .get_matches();

    let (name_values, child_cmdline) = parse_args(&matches);

    if child_cmdline.is_empty() {
        eprintln!("Error: No command provided to run.");
        process::exit(2);
    }

    let mut child = process::Command::new(&child_cmdline[0]);
    child.args(&child_cmdline[1..]);

    if matches.get_flag("ignore-environment") {
        child.env_clear();
    }
    for var in matches.get_many::<String>("unset").unwrap_or_default() {
        child.env_remove(var);
    }
    child.envs(name_values);

    let status = child
        .spawn()
        .expect("Failed to execute command")
        .wait()
        .expect("Failed to wait on child process")
        .code()
        .unwrap_or(1); // TODO: Should some higher number be used?

    process::exit(status);
}

fn parse_args(matches: &ArgMatches) -> (Vec<(String, String)>, Vec<String>) {
    let mut name_values = Vec::new();
    let mut command = Vec::new();
    let mut in_command = false;

    for arg in matches.get_many::<String>("args").unwrap_or_default() {
        if in_command {
            command.push(arg.clone());
        } else if let Some((name, value)) = arg.split_once('=') {
            name_values.push((name.to_string(), value.to_string()));
        } else {
            in_command = true;
            command.push(arg.clone());
        }
    }

    (name_values, command)
}
