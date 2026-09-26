use thiserror::Error;

use crate::records::RecordKind;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error when trying to convert magic.dot: {exit_status}")]
    ExitStatusError {
        exit_status: std::process::ExitStatus,
    },

    #[error("no {record_type} record matched {field}")]
    NotFound {
        record_type: &'static str,
        field: String,
    },

    #[error("expected a {expected} record, got {actual:?}")]
    WrongRecordType {
        expected: &'static str,
        actual: RecordKind,
    },

    #[error("record conversion failed: {0}")]
    RecordConversion(#[from] RecordConversionError),

    #[error("cycle detected in {kind} graph at ID {id}")]
    GraphCycle { kind: &'static str, id: i64 },

    #[error("Unexpected argument {args:?} found.")]
    UnexpectedArgs { args: Vec<String> },

    #[error("Usage: wiki-tools \"<spell name>\"")]
    NoArgs,

    #[error("{message}")]
    WorkerPanicked { message: String },
}

#[derive(Debug, Error)]
pub enum RecordConversionError {
    #[error("expected a Magic record, got {0:?}")]
    ExpectedMagic(RecordKind),

    #[error("expected a Bullet record, got {0:?}")]
    ExpectedBullet(RecordKind),

    #[error("expected an AtkParamPC record, got {0:?}")]
    ExpectedAtkParamPc(RecordKind),
}
