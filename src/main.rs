//! Runs a program with a modified environment.

use std::str::FromStr;

use clap::{Arg, ArgAction};

#[derive(Clone, Debug)]
struct EnvVar {
    name: String,
    value: String,
}

impl FromStr for EnvVar {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            Some((name, value)) if !name.is_empty() => Ok(Self {
                name: name.to_string(),
                value: value.to_string(),
            }),
            Some(_) => Err(format!("NAME must be nonempty in NAME=VALUE (got {s:?})")),
            None => Err(format!("Expected NAME=VALUE (got {s:?})")),
        }
    }
}

fn main() {
    let matches = clap::Command::new("useenv")
        .version("0.1.0")
        .about("Run a program with a modified environment")
        .arg(
            Arg::new("ignore-environment")
                .short('i')
                .long("ignore-environment")
                .help("Clear all environment variables not explicitly set")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("unset")
                .short('u')
                .long("unset")
                .help("Clear a specific variable from the environment")
                .action(ArgAction::Append)
                .num_args(1),
        )
        .arg(
            Arg::new("NAME=VALUE")
                .help("Set an environment variable of the given name and value")
                .num_args(0..)
                .value_name("NAME=VALUE")
                .value_parser(EnvVar::from_str),
        )
        .get_matches();

    if matches.get_flag("ignore-environment") {
        println!("Clearing all environment variables not explicitly set");
    }

    if let Some(unset_vars) = matches.get_many::<String>("unset") {
        for var in unset_vars {
            println!("Unsetting variable {:?}", var);
        }
    }

    if let Some(env_vars) = matches.get_many::<EnvVar>("NAME=VALUE") {
        for var in env_vars {
            println!("Setting variable {:?} to value {:?}", var.name, var.value);
        }
    }
}
