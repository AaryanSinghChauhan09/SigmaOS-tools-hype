//! Slide model — one slide in a hype deck.

use crate::markdown::Block;

/// Slide layout kind (inferred from content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideKind {
    /// Title slide (single H1).
    Title,
    /// Content slide.
    Content,
    /// Section divider.
    Section,
    /// Closing slide.
    Closing,
}

/// A single slide.
#[derive(Debug, Clone)]
pub struct Slide {
    /// Slide number (1-based).
    pub number: usize,
    /// Parsed blocks.
    pub blocks: Vec<Block>,
    /// Presenter notes extracted from `???`/Note blocks.
    pub notes: Vec<String>,
    /// Layout kind.
    pub kind: SlideKind,
}

impl Slide {
    /// Extract plain-text title (first heading's text).
    pub fn title(&self) -> String {
        for b in &self.blocks {
            if let Block::Heading { inlines, .. } = b {
                return inlines
                    .iter()
                    .map(|i| match i {
                        crate::markdown::Inline::Text(s)
                        | crate::markdown::Inline::Bold(s)
                        | crate::markdown::Inline::Italic(s)
                        | crate::markdown::Inline::Code(s) => s.clone(),
                        crate::markdown::Inline::Link { text, .. } => text.clone(),
                        crate::markdown::Inline::Image { alt, .. } => alt.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join("");
            }
        }
        String::new()
    }

    /// Plain-text body (all non-heading blocks rendered as text).
    pub fn text(&self) -> String {
        let mut out = String::new();
        for b in &self.blocks {
            match b {
                Block::Paragraph(inlines) | Block::Quote(inlines) => {
                    for i in inlines {
                        match i {
                            crate::markdown::Inline::Text(s)
                            | crate::markdown::Inline::Bold(s)
                            | crate::markdown::Inline::Italic(s)
                            | crate::markdown::Inline::Code(s) => out.push_str(s),
                            crate::markdown::Inline::Link { text, .. } => out.push_str(text),
                            crate::markdown::Inline::Image { alt, .. } => out.push_str(alt),
                        }
                    }
                    out.push('\n');
                }
                Block::List(items) | Block::OrderedList(items) => {
                    for item in items {
                        out.push_str("- ");
                        for i in item {
                            if let crate::markdown::Inline::Text(s) = i {
                                out.push_str(s);
                            }
                        }
                        out.push('\n');
                    }
                }
                Block::CodeBlock { code, .. } => {
                    out.push_str(code);
                    out.push('\n');
                }
                Block::Heading { inlines, .. } => {
                    for i in inlines {
                        if let crate::markdown::Inline::Text(s) = i {
                            out.push_str(s);
                        }
                    }
                    out.push('\n');
                }
                _ => {}
            }
        }
        out
    }
}
