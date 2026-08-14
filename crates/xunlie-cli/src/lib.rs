#![forbid(unsafe_code)]

//! Stable command-line boundary for Xunlie.
//!
//! Human-readable output is written to stdout on success and stderr on failure.
//! `--format json` follows the same stream rule and emits exactly one JSON object,
//! making it safe to consume from CI without mixing diagnostics and logs.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{ColorChoice, Parser, Subcommand, ValueEnum, error::ErrorKind};
use serde::Serialize;
use xunlie_domain::{ContractIr, Diagnostic};
use xunlie_engine::{SourceDocument, compile_sources};

/// Process exit codes are part of the public CLI contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitStatus {
    /// The requested operation completed successfully.
    Success = 0,
    /// Arguments or command selection were invalid.
    Usage = 2,
    /// An input file could not be read.
    InputIo = 10,
    /// Source compilation failed without producing partial `ContractIR`.
    CompileFailed = 11,
    /// A persisted `ContractIR` document was invalid.
    ContractInvalid = 12,
    /// A compiled artifact could not be written.
    OutputIo = 13,
    /// CLI infrastructure, such as its output stream, failed.
    Internal = 70,
}

impl ExitStatus {
    /// Returns the stable numeric process code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

impl From<ExitStatus> for ExitCode {
    fn from(value: ExitStatus) -> Self {
        Self::from(value.code())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum OutputFormat {
    /// Concise messages intended for a person at a terminal.
    #[default]
    Human,
    /// A single `xunlie.cli.result/v1` object intended for automation.
    Json,
}

#[derive(Debug, Parser)]
#[command(
    name = "xunlie",
    version,
    about = "Compile and validate deterministic Xunlie contracts",
    disable_help_subcommand = true,
    color = ColorChoice::Never
)]
struct Cli {
    /// Output format for command results. Logs, if enabled in future versions, remain on stderr.
    #[arg(long, value_enum, default_value_t, global = true)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Compile a source document into canonical ContractIR JSON.
    Compile {
        /// Source document to compile.
        input: PathBuf,

        /// Destination for canonical ContractIR JSON.
        #[arg(long, short = 'o', value_name = "FILE")]
        out: PathBuf,
    },

    /// Validate an existing ContractIR document.
    Validate {
        /// ContractIR JSON document to validate.
        contract: PathBuf,
    },
}

#[derive(Debug)]
struct CompiledContract {
    canonical_json: String,
    content_digest: String,
    artifact_digest: String,
}

#[derive(Debug)]
struct ValidatedContract {
    content_digest: String,
    artifact_digest: String,
}

trait ContractEngine {
    fn compile(&self, identity: &str, source: &str) -> Result<CompiledContract, EngineFailure>;
    fn validate(&self, bytes: &[u8]) -> Result<ValidatedContract, EngineFailure>;
}

#[derive(Debug)]
struct EngineFailure {
    message: String,
    diagnostics: Vec<Diagnostic>,
}

impl EngineFailure {
    fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct ProductionEngine;

impl ContractEngine for ProductionEngine {
    fn compile(&self, identity: &str, source: &str) -> Result<CompiledContract, EngineFailure> {
        let source = SourceDocument::new(identity, 0, source);
        let contract = compile_sources(vec![source]).map_err(|error| EngineFailure {
            message: error.to_string(),
            diagnostics: error.diagnostics().to_vec(),
        })?;

        Ok(CompiledContract {
            canonical_json: contract
                .canonical_json()
                .map_err(|error| EngineFailure::message(error.to_string()))?,
            content_digest: contract.content_digest().to_string(),
            artifact_digest: contract.artifact_digest().to_string(),
        })
    }

    fn validate(&self, bytes: &[u8]) -> Result<ValidatedContract, EngineFailure> {
        let contract: ContractIr = serde_json::from_slice(bytes)
            .map_err(|error| EngineFailure::message(format!("invalid ContractIR JSON: {error}")))?;
        contract.validate().map_err(|diagnostics| EngineFailure {
            message: diagnostics.first().map_or_else(
                || "ContractIR invariants failed".to_owned(),
                |item| item.message.clone(),
            ),
            diagnostics,
        })?;

        Ok(ValidatedContract {
            content_digest: contract.content_digest().to_string(),
            artifact_digest: contract.artifact_digest().to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessOutput {
    schema_version: &'static str,
    command: &'static str,
    status: &'static str,
    input: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<PathBuf>,
    content_digest: String,
    artifact_digest: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorOutput<'a> {
    schema_version: &'static str,
    command: &'a str,
    status: &'static str,
    code: &'a str,
    message: &'a str,
    exit_code: u8,
    #[serde(skip_serializing_if = "diagnostics_are_empty")]
    diagnostics: &'a [Diagnostic],
}

fn diagnostics_are_empty(diagnostics: &&[Diagnostic]) -> bool {
    diagnostics.is_empty()
}

#[derive(Debug)]
struct CommandError {
    exit: ExitStatus,
    code: &'static str,
    command: &'static str,
    message: String,
    diagnostics: Vec<Diagnostic>,
}

impl CommandError {
    fn new(
        exit: ExitStatus,
        code: &'static str,
        command: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            exit,
            code,
            command,
            message: message.into(),
            diagnostics: Vec::new(),
        }
    }

    fn with_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}

/// Run the CLI against process stdout/stderr and return a stable process status.
#[must_use]
pub fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run_with_io(args, &mut io::stdout().lock(), &mut io::stderr().lock()).into()
}

/// Injectable I/O entry point used by tests and embedders.
#[must_use]
pub fn run_with_io<I, T>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> ExitStatus
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run_with_engine(args, stdout, stderr, &ProductionEngine)
}

fn run_with_engine<I, T>(
    args: I,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    engine: &dyn ContractEngine,
) -> ExitStatus
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let requested_format = requested_output_format(&args);
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let is_help = matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            );
            let exit = if is_help {
                ExitStatus::Success
            } else {
                ExitStatus::Usage
            };
            let result = if is_help {
                write!(stdout, "{error}")
            } else if requested_format == OutputFormat::Human {
                write!(stderr, "{error}")
            } else {
                write_error(
                    stderr,
                    OutputFormat::Json,
                    &CommandError::new(
                        ExitStatus::Usage,
                        "XUNLIE-E002",
                        "cli",
                        error.to_string().trim().to_owned(),
                    ),
                )
            };
            if result.is_err() {
                return ExitStatus::Internal;
            }
            return exit;
        }
    };

    match execute(&cli.command, engine) {
        Ok(success) => {
            if write_success(stdout, cli.format, &success).is_err() {
                ExitStatus::Internal
            } else {
                ExitStatus::Success
            }
        }
        Err(error) => {
            if write_error(stderr, cli.format, &error).is_err() {
                ExitStatus::Internal
            } else {
                error.exit
            }
        }
    }
}

fn requested_output_format(args: &[OsString]) -> OutputFormat {
    for (index, argument) in args.iter().enumerate() {
        let argument = argument.to_string_lossy();
        if argument == "--format" {
            if args
                .get(index + 1)
                .is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case("json"))
            {
                return OutputFormat::Json;
            }
        } else if argument.eq_ignore_ascii_case("--format=json") {
            return OutputFormat::Json;
        }
    }
    OutputFormat::Human
}

fn execute(command: &Command, engine: &dyn ContractEngine) -> Result<SuccessOutput, CommandError> {
    match command {
        Command::Compile { input, out } => {
            let source = read(input, "compile")?;
            let identity = portable_identity(input);
            let compiled = engine.compile(&identity, &source).map_err(|failure| {
                CommandError::new(
                    ExitStatus::CompileFailed,
                    "XUNLIE-E011",
                    "compile",
                    format!(
                        "could not compile '{}': {}",
                        input.display(),
                        failure.message
                    ),
                )
                .with_diagnostics(failure.diagnostics)
            })?;

            fs::write(out, compiled.canonical_json.as_bytes()).map_err(|error| {
                CommandError::new(
                    ExitStatus::OutputIo,
                    "XUNLIE-E013",
                    "compile",
                    format!("could not write '{}': {error}", out.display()),
                )
            })?;

            Ok(SuccessOutput {
                schema_version: "xunlie.cli.result/v1",
                command: "compile",
                status: "ok",
                input: input.clone(),
                output: Some(out.clone()),
                content_digest: compiled.content_digest,
                artifact_digest: compiled.artifact_digest,
            })
        }
        Command::Validate { contract } => {
            let bytes = fs::read(contract).map_err(|error| {
                CommandError::new(
                    ExitStatus::InputIo,
                    "XUNLIE-E010",
                    "validate",
                    format!("could not read '{}': {error}", contract.display()),
                )
            })?;
            let validated = engine.validate(&bytes).map_err(|failure| {
                CommandError::new(
                    ExitStatus::ContractInvalid,
                    "XUNLIE-E012",
                    "validate",
                    format!(
                        "contract '{}' is invalid: {}",
                        contract.display(),
                        failure.message
                    ),
                )
                .with_diagnostics(failure.diagnostics)
            })?;

            Ok(SuccessOutput {
                schema_version: "xunlie.cli.result/v1",
                command: "validate",
                status: "ok",
                input: contract.clone(),
                output: None,
                content_digest: validated.content_digest,
                artifact_digest: validated.artifact_digest,
            })
        }
    }
}

fn read(path: &Path, command: &'static str) -> Result<String, CommandError> {
    fs::read_to_string(path).map_err(|error| {
        CommandError::new(
            ExitStatus::InputIo,
            "XUNLIE-E010",
            command,
            format!("could not read '{}': {error}", path.display()),
        )
    })
}

fn portable_identity(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_success(
    writer: &mut dyn Write,
    format: OutputFormat,
    output: &SuccessOutput,
) -> io::Result<()> {
    match format {
        OutputFormat::Human => {
            if let Some(path) = output.output.as_deref() {
                writeln!(
                    writer,
                    "compiled {} -> {}",
                    output.input.display(),
                    path.display()
                )?;
            } else {
                writeln!(writer, "valid {}", output.input.display())?;
            }
            writeln!(writer, "content digest: {}", output.content_digest)?;
            writeln!(writer, "artifact digest: {}", output.artifact_digest)
        }
        OutputFormat::Json => {
            serde_json::to_writer(&mut *writer, output)?;
            writeln!(writer)
        }
    }
}

fn write_error(
    writer: &mut dyn Write,
    format: OutputFormat,
    error: &CommandError,
) -> io::Result<()> {
    match format {
        OutputFormat::Human => {
            writeln!(writer, "{}: {}", error.code, error.message)?;
            for diagnostic in &error.diagnostics {
                writeln!(writer, "  {}: {}", diagnostic.code, diagnostic.message)?;
            }
            Ok(())
        }
        OutputFormat::Json => {
            serde_json::to_writer(
                &mut *writer,
                &ErrorOutput {
                    schema_version: "xunlie.cli.result/v1",
                    command: error.command,
                    status: "error",
                    code: error.code,
                    message: &error.message,
                    exit_code: error.exit.code(),
                    diagnostics: &error.diagnostics,
                },
            )?;
            writeln!(writer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct StubEngine;

    impl ContractEngine for StubEngine {
        fn compile(
            &self,
            _identity: &str,
            source: &str,
        ) -> Result<CompiledContract, EngineFailure> {
            if source == "bad" {
                return Err(EngineFailure::message("fixture rejected"));
            }
            Ok(CompiledContract {
                canonical_json: "{\"schemaVersion\":\"xunlie.contract/v1\"}\n".into(),
                content_digest: "sha256:fixture".into(),
                artifact_digest: "sha256:artifact-fixture".into(),
            })
        }

        fn validate(&self, bytes: &[u8]) -> Result<ValidatedContract, EngineFailure> {
            if bytes == b"bad" {
                return Err(EngineFailure::message("fixture invalid"));
            }
            Ok(ValidatedContract {
                content_digest: "sha256:fixture".into(),
                artifact_digest: "sha256:artifact-fixture".into(),
            })
        }
    }

    #[test]
    fn usage_errors_have_stable_exit_code() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let status = run_with_engine(["xunlie", "unknown"], &mut stdout, &mut stderr, &StubEngine);

        assert_eq!(status, ExitStatus::Usage);
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }

    #[test]
    fn usage_error_respects_json_format() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let status = run_with_engine(
            ["xunlie", "unknown", "--format", "json"],
            &mut stdout,
            &mut stderr,
            &StubEngine,
        );

        assert_eq!(status, ExitStatus::Usage);
        assert!(stdout.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(value["code"], "XUNLIE-E002");
        assert_eq!(value["exitCode"], 2);
    }

    #[test]
    fn help_is_successful_and_uses_stdout() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let status = run_with_engine(["xunlie", "--help"], &mut stdout, &mut stderr, &StubEngine);

        assert_eq!(status, ExitStatus::Success);
        assert!(String::from_utf8(stdout).unwrap().contains("Usage:"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn validation_failure_is_machine_readable() {
        let directory = tempfile::tempdir().unwrap();
        let contract = directory.path().join("contract.json");
        fs::write(&contract, "bad").unwrap();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let args = vec![
            OsString::from("xunlie"),
            OsString::from("validate"),
            contract.into_os_string(),
            OsString::from("--format"),
            OsString::from("json"),
        ];

        let status = run_with_engine(args, &mut stdout, &mut stderr, &StubEngine);

        assert_eq!(status, ExitStatus::ContractInvalid);
        assert!(stdout.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(value["code"], "XUNLIE-E012");
        assert_eq!(value["exitCode"], 12);
    }

    #[test]
    fn compile_writes_contract_and_reports_digest() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("source.json");
        let output = directory.path().join("contract.json");
        fs::write(&input, "good").unwrap();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let args = vec![
            OsString::from("xunlie"),
            OsString::from("compile"),
            input.into_os_string(),
            OsString::from("--out"),
            output.clone().into_os_string(),
            OsString::from("--format"),
            OsString::from("json"),
        ];

        let status = run_with_engine(args, &mut stdout, &mut stderr, &StubEngine);

        assert_eq!(status, ExitStatus::Success);
        assert!(stderr.is_empty());
        assert_eq!(
            fs::read_to_string(output).unwrap(),
            "{\"schemaVersion\":\"xunlie.contract/v1\"}\n"
        );
        let value: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["contentDigest"], "sha256:fixture");
        assert_eq!(value["artifactDigest"], "sha256:artifact-fixture");
    }

    #[test]
    fn source_identity_uses_portable_separators() {
        assert_eq!(
            portable_identity(Path::new(r"examples\minimal-source.json")),
            "examples/minimal-source.json"
        );
    }
}
