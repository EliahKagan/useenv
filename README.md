# useenv - Run a program with a modified environment

This is kind of like `env` but cross-platform.

This is inspired by the Unix `env` command but not equivalent to it. This is
due both to non-portable aspects related to its typical implementation in
terms of `exec`, and to the currently very rough condition this code is in.

Depending on your goals, you may prefer to use the MSYS2 `env` command. Part of
the goal here is to avoid affecting the environment in any Unix-ish way.

## License

[0BSD](LICENSE)

## Usage

```text
useenv [{option | nameval}...] command [args...]
```

Options:

- `-i` or `--ignore-environment`: Clear all environment variables, passing only those that are passed as name-value pairs.
- `-u NAME` or `--unset NAME`: Clear the variable named `NAME`, passing it only if it is also specified as a name-value pair.

Name-value pairs:

- A name followed by an equals sign and zero or more characters.
- The name itself must usually be at least one character.
- Explicitly specified empty environment variables are passed as empty variables. They are not unset, even on Windows where shells often use such a syntax to unset them.

The first argument that is neither an option (or `-u` operand) nor a `NAME=value` pair is taken to be the beginning of the command to be run. Subsequent arguments are the argument to that command.

Options and `NAME=value` pairs may be interleaved arbitrarily, prior to the first command to be run.

## Limitations

This does not do anything in a platform-specific way, except to the extent the Rust standard library does so. As such, it inherently does not meet the expectations of `env`.

However, it would be possible to get much closer, even on Windows. This does not attempt to simulate however the child process exited, beyond returning the same error code the child process returns if it terminates normally.

The goal here was just to make a simple cross-platform program that conceptually works the same way on all systems and allows environment variables to be customized in a way that didn't involve the complex rules of how shells run subprocesses.

The limited purpose of this program shows. For example, it does not support full Unix-style option syntax.
