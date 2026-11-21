//! Integration tests for sigmaos-tools-hype.

use sigmaos_tools_hype::{Deck, parse_markdown, parse_inline, Inline, Block};

#[test]
fn test_parse_deck() {
    let md = "# Welcome\n\nIntro text\n\n---\n\n## Agenda\n\n- one\n- two\n\n---\n\n# Thanks\n\n??? say goodbye\n";
    let d = Deck::parse(md).unwrap();
    assert_eq!(d.len(), 3);
    assert_eq!(d.slides[0].title(), "Welcome");
    assert_eq!(d.slides[2].notes.len(), 1);
}

#[test]
fn test_navigation() {
    let mut d = Deck::parse("# A\n\n---\n\n# B\n").unwrap();
    assert!(d.next());
    assert_eq!(d.current, 1);
    assert!(!d.next());
    assert!(d.prev());
}

#[test]
fn test_inline_formatting() {
    let inlines = parse_inline("**bold** and *italic* and `code`");
    assert!(inlines.iter().any(|i| matches!(i, Inline::Bold(_))));
    assert!(inlines.iter().any(|i| matches!(i, Inline::Italic(_))));
    assert!(inlines.iter().any(|i| matches!(i, Inline::Code(_))));
}

#[test]
fn test_markdown_blocks() {
    let blocks = parse_markdown("# Title\n\n- a\n- b\n\n```rust\nfn main() {}\n```\n");
    assert!(blocks.iter().any(|b| matches!(b, Block::Heading { level: 1, .. })));
    assert!(blocks.iter().any(|b| matches!(b, Block::List(_))));
    assert!(blocks.iter().any(|b| matches!(b, Block::CodeBlock { lang, .. } if lang == "rust")));
}
