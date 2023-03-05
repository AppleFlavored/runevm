use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClassFileError {
    #[error("reached end of stream")]
    IoError(#[from] std::io::Error),
    #[error("invalid magic (expected CAFEBABE but received {0:X})")]
    InvalidMagic(u32),
    #[error("invalid tag `{0}` found in constant pool")]
    InvalidTag(u8),
    #[error("unexpected constant at index `{0}`")]
    InvalidConstant(u16),
    #[error("found invalid attribute `{0}`")]
    InvalidAttribute(String),
}
