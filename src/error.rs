//! Error types for the hype component.

/// Errors in deck parsing/navigation.
#[derive(Debug, thiserror::Error)]
pub enum HypeError {
    /// Deck file could not be read.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Deck has no slides.
    #[error("empty deck")]
    EmptyDeck,

    /// Slide index out of range.
    #[error("slide index out of range: {0}")]
    BadIndex(usize),
}
