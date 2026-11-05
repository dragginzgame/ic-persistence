///
/// `CollectionError`
///
/// A rejected collection operation. Encoding, validation and capacity errors
/// precede writes; backing panics still follow the host's failure contract.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CollectionError {
    /// Application encoding or declared size is invalid.
    #[error("{0}")]
    Codec(#[from] crate::storable::CodecError),
    /// The persisted record or collection structure is invalid.
    #[error("invalid collection: {0}")]
    Corrupt(&'static str),
    /// The bytes belong to another format.
    #[error("unrecognized collection format")]
    Format,
    /// Record integrity verification failed.
    #[error("collection checksum mismatch")]
    Checksum,
    /// A record, arithmetic operation or allocation exceeds its resource budget.
    #[error("collection resource budget exceeded")]
    Budget,
    /// Capacity could not be reserved.
    #[error("collection growth refused")]
    Growth,
    /// Creation was attempted in nonempty memory.
    #[error("collection memory is not empty")]
    NotEmpty,
    /// A prior backing failure prevents further access through this owner.
    #[error("collection owner is poisoned")]
    Poisoned,
    /// An index or range is invalid.
    #[error("invalid collection index or range")]
    Bounds,
    /// Bounded Wire encoding or decoding failed.
    #[error("collection Wire record: {0}")]
    Wire(#[from] ic_wire::Error),
    /// Cell envelope admission failed before reading its payload.
    #[error("{0}")]
    Cell(#[from] crate::StableCellPayloadError),
}

pub(super) type Result<T> = std::result::Result<T, CollectionError>;

pub(super) const fn ensure(condition: bool, error: CollectionError) -> Result<()> {
    if condition { Ok(()) } else { Err(error) }
}
