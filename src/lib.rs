//! SigmaOS tools-hype component.
//!
//! Rust implementation of `omacom/hype`: Omarchy's Markdown presentation
//! tool — decks defined as `.md` files with `---` slide separators and a
//! visual slide editor.
//!
//! This port implements the deck model: markdown→slide parsing (headers,
//! lists, code blocks, images), slide navigation state, and presenter
//! notes — the logic upstream keeps in QML/JS models.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod deck;
pub mod error;
pub mod markdown;
pub mod slide;

pub use deck::Deck;
pub use slide::{Slide, SlideKind};
pub use error::HypeError;
pub use markdown::{Block, Inline, parse_inline, parse_markdown};
