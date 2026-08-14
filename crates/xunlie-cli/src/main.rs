#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    xunlie_cli::run(std::env::args_os())
}
