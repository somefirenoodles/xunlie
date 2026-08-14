#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const FORBIDDEN_DOMAIN_TOKENS: &[&str] = &[
    "std::fs",
    "std::net",
    "std::process",
    "std::time::SystemTime",
    "rand::",
    "reqwest::",
    "tokio::",
    "ureq::",
    "unsafe {",
    "unsafe fn",
];

fn main() -> ExitCode {
    let command = env::args().nth(1).unwrap_or_else(|| "help".to_owned());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live directly below the workspace root");

    let result = match command.as_str() {
        "quality" => quality(root),
        "architecture" => architecture(root),
        "traceability" | "governance" => governance(root),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        unknown => Err(format!("unknown xtask command '{unknown}'")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

fn quality(root: &Path) -> Result<(), String> {
    governance(root)?;
    architecture(root)?;
    cargo(root, ["fmt", "--all", "--", "--check"])?;
    cargo(
        root,
        [
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    cargo(
        root,
        [
            "test",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
        ],
    )
}

fn governance(root: &Path) -> Result<(), String> {
    let script = root.join("scripts/validate_quality_system.py");
    for python in python_candidates() {
        match Command::new(&python)
            .arg(&script)
            .current_dir(root)
            .status()
        {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => {
                return Err(format!(
                    "quality-system validator failed with {status} using {}",
                    python.to_string_lossy()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("could not run Python validator: {error}")),
        }
    }
    Err("Python 3 was not found (tried python3 and python)".to_owned())
}

fn architecture(root: &Path) -> Result<(), String> {
    let source_root = root.join("crates/xunlie-domain/src");
    if !source_root.is_dir() {
        return Err(format!(
            "domain source directory is missing: {}",
            source_root.display()
        ));
    }

    let mut rust_files = Vec::new();
    collect_rust_files(&source_root, &mut rust_files)?;
    if rust_files.is_empty() {
        return Err("domain crate contains no Rust source files".to_owned());
    }

    let mut violations = Vec::new();
    for path in rust_files {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        for token in FORBIDDEN_DOMAIN_TOKENS {
            if source.contains(token) {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                violations.push(format!(
                    "{} contains forbidden token {token:?}",
                    relative.display()
                ));
            }
        }
    }

    if violations.is_empty() {
        println!("ARCHITECTURE FITNESS: PASS");
        Ok(())
    } else {
        Err(format!(
            "architecture violations:\n- {}",
            violations.join("\n- ")
        ))
    }
}

fn collect_rust_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("could not read {}: {error}", directory.display()))?
    {
        let path = entry
            .map_err(|error| format!("could not inspect {}: {error}", directory.display()))?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, output)?;
        } else if path.extension() == Some(OsStr::new("rs")) {
            output.push(path);
        }
    }
    Ok(())
}

fn cargo<const N: usize>(root: &Path, arguments: [&str; N]) -> Result<(), String> {
    let executable = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    println!("+ cargo {}", arguments.join(" "));
    let status = Command::new(executable)
        .args(arguments)
        .current_dir(root)
        .status()
        .map_err(|error| format!("could not run cargo: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo command failed with {status}"))
    }
}

#[cfg(windows)]
fn python_candidates() -> [std::ffi::OsString; 2] {
    ["python".into(), "python3".into()]
}

#[cfg(not(windows))]
fn python_candidates() -> [std::ffi::OsString; 2] {
    ["python3".into(), "python".into()]
}

fn print_help() {
    println!(
        "Xunlie workspace tasks\n\n  cargo xtask quality       Run all pre-PR checks\n  cargo xtask architecture  Check trusted-core boundaries\n  cargo xtask traceability  Validate requirements and governance\n"
    );
}
