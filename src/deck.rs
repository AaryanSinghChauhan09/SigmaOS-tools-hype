//! Deck model — markdown file → slides + navigation.

use std::path::Path;

use crate::markdown::{Block, parse_markdown};
use crate::slide::{Slide, SlideKind};
use crate::HypeError;

/// A presentation deck.
#[derive(Debug, Clone)]
pub struct Deck {
    /// Source file path.
    pub path: Option<std::path::PathBuf>,
    /// Slides.
    pub slides: Vec<Slide>,
    /// Current slide index (0-based).
    pub current: usize,
}

impl Deck {
    /// Parse a deck from markdown text.
    ///
    /// Slides are separated by `---` lines (horizontal rules); a leading
    /// `---` at the very start is ignored (front-matter style).
    pub fn parse(text: &str) -> Result<Self, HypeError> {
        // Split on lines that are exactly "---".
        let mut slide_texts: Vec<String> = Vec::new();
        let mut cur = String::new();
        for line in text.lines() {
            if line.trim() == "---" {
                if !cur.trim().is_empty() {
                    slide_texts.push(std::mem::take(&mut cur));
                }
            } else {
                cur.push_str(line);
                cur.push('\n');
            }
        }
        if !cur.trim().is_empty() {
            slide_texts.push(cur);
        }
        if slide_texts.is_empty() {
            return Err(HypeError::EmptyDeck);
        }
        let mut slides = Vec::new();
        for (i, st) in slide_texts.iter().enumerate() {
            let mut blocks = parse_markdown(st);
            // Extract presenter notes into slide.notes, keep blocks.
            let mut notes = Vec::new();
            blocks.retain(|b| {
                if let Block::Note(inlines) = b {
                    notes.push(inlines.iter().map(|i| match i {
                        crate::markdown::Inline::Text(s)
                        | crate::markdown::Inline::Bold(s)
                        | crate::markdown::Inline::Italic(s)
                        | crate::markdown::Inline::Code(s) => s.clone(),
                        _ => String::new(),
                    }).collect::<String>());
                    false
                } else {
                    true
                }
            });
            let kind = infer_kind(&blocks);
            slides.push(Slide {
                number: i + 1,
                blocks,
                notes,
                kind,
            });
        }
        Ok(Self { path: None, slides, current: 0 })
    }

    /// Load a deck from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, HypeError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;
        let mut deck = Self::parse(&text)?;
        deck.path = Some(path.to_path_buf());
        Ok(deck)
    }

    /// Current slide.
    pub fn current_slide(&self) -> Option<&Slide> {
        self.slides.get(self.current)
    }

    /// Total slides.
    pub fn len(&self) -> usize {
        self.slides.len()
    }

    /// Whether deck is empty.
    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }

    /// Go to next slide.
    pub fn next(&mut self) -> bool {
        if self.current + 1 < self.slides.len() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// Go to previous slide.
    pub fn prev(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false
        }
    }

    /// Jump to slide index (0-based).
    pub fn goto(&mut self, index: usize) -> Result<(), HypeError> {
        if index < self.slides.len() {
            self.current = index;
            Ok(())
        } else {
            Err(HypeError::BadIndex(index))
        }
    }

    /// Progress fraction (0.0–1.0).
    pub fn progress(&self) -> f64 {
        if self.slides.is_empty() {
            0.0
        } else {
            self.current as f64 / (self.slides.len() - 1).max(1) as f64
        }
    }

    /// Slide titles for overview mode.
    pub fn titles(&self) -> Vec<String> {
        self.slides.iter().map(|s| s.title()).collect()
    }
}

fn infer_kind(blocks: &[Block]) -> SlideKind {
    let headings = blocks.iter().filter(|b| matches!(b, Block::Heading { .. })).count();
    let h1 = blocks.iter().any(|b| matches!(b, Block::Heading { level: 1, .. }));
    if headings == 1 && h1 && blocks.len() <= 3 {
        SlideKind::Title
    } else if headings == 1 && blocks.len() <= 2 {
        SlideKind::Section
    } else {
        SlideKind::Content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_two_slides() {
        let text = "# Title\n\nHello\n\n---\n\n## Slide 2\n\n- a\n- b\n";
        let d = Deck::parse(text).unwrap();
        assert_eq!(d.len(), 2);
        assert_eq!(d.slides[0].kind, SlideKind::Title);
        assert_eq!(d.slides[1].title(), "Slide 2");
    }

    #[test]
    fn test_navigation() {
        let text = "# A\n\n---\n\n# B\n\n---\n\n# C\n";
        let mut d = Deck::parse(text).unwrap();
        assert!(d.next());
        assert_eq!(d.current, 1);
        assert!(d.next());
        assert!(!d.next()); // at end
        assert!(d.prev());
        assert_eq!(d.current, 1);
    }

    #[test]
    fn test_notes_extracted() {
        let text = "# T\n\ncontent\n\n??? remember this\n";
        let d = Deck::parse(text).unwrap();
        assert_eq!(d.slides[0].notes.len(), 1);
        assert!(d.slides[0].notes[0].contains("remember"));
    }

    #[test]
    fn test_empty_deck() {
        assert!(Deck::parse("").is_err());
        assert!(Deck::parse("   \n\n").is_err());
    }
}
