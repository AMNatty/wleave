use miette::{Diagnostic, NamedSource, SourceOffset};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum WError {
    #[error("Failed to load the specified configuration file {0} as it does not exist")]
    SpecifiedPathNotAFile(PathBuf),
    #[error("Failed to find the configuration file {0} in the search path")]
    FileNotInSearchPath(PathBuf),
    #[error("An error has occurred while reading file {0}: {1}")]
    IoError(PathBuf, std::io::Error),
    #[error("JSON parsing failed")]
    #[diagnostic(code(wleave::parse_failed))]
    FileParseFailed(
        #[source_code] NamedSource<String>,
        #[label("The parser failed here")] SourceOffset,
        #[source] serde_json::Error,
    ),
}
