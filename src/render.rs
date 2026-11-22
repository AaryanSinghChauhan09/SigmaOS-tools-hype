//! Terminal renderer — render slides to ANSI text for CLI/TUI playback.
//!
//! Upstream hype is a QML presentation editor; this port renders the
//! same `Slide`/`Block`/`Inline` model to a terminal frame: centered
//! title for Title/Section/Closing slides, left-aligned content for
//! Content slides, with ANSI bold/italic/code styling.

use crate::deck::Deck;
use crate::markdown::{Block, Inline};
use crate::slide::{Slide, SlideKind};

/// ANSI SGR codes.
mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const CYAN: &str = "\x1b[36m";
    pub const YELLOW: &str = "\x1b[33m";
}

/// Render options.
#[derive(Debug, Clone)]
pub struct RenderOpts {
    /// Terminal width in columns.
    pub width: usize,
    /// Terminal height in rows.
    pub height: usize,
    /// Show slide number/progress footer.
    pub show_footer: bool,
    /// Use ANSI colors (false = plain text).
    pub color: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self { width: 80, height: 24, show_footer: true, color: true }
    }
}

/// Render a single slide to terminal lines.
pub fn render_slide(slide: &Slide, deck: &Deck, opts: &RenderOpts) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    match slide.kind {
        SlideKind::Title => render_title(slide, opts, &mut lines),
        SlideKind::Section => render_section(slide, opts, &mut lines),
        SlideKind::Closing => render_closing(slide, opts, &mut lines),
        SlideKind::Content => render_content(slide, opts, &mut lines),
    }
    if opts.show_footer {
        lines.push(footer(deck, opts));
    }
    // Pad to height.
    while lines.len() < opts.height {
        lines.push(String::new());
    }
    lines.truncate(opts.height);
    lines
}

fn render_title(slide: &Slide, opts: &RenderOpts, out: &mut Vec<String>) {
    // Center vertically.
    let pad_top = opts.height / 3;
    for _ in 0..pad_top {
        out.push(String::new());
    }
    let title = slide.title();
    out.push(center(&title, opts.width));
    // Subtitle = first paragraph.
    if let Some(Block::Paragraph(inlines)) = slide.blocks.iter().find(|b| matches!(b, Block::Paragraph(_))) {
        let sub = inline_text(inlines);
        out.push(center(&dim(&sub, opts), opts.width));
    }
}

fn render_section(slide: &Slide, opts: &RenderOpts, out: &mut Vec<String>) {
    let pad_top = opts.height / 2;
    for _ in 0..pad_top {
        out.push(String::new());
    }
    let title = slide.title();
    out.push(center(&cyan(&title, opts), opts.width));
}

fn render_closing(slide: &Slide, opts: &RenderOpts, out: &mut Vec<String>) {
    let pad_top = opts.height / 2;
    for _ in 0..pad_top {
        out.push(String::new());
    }
    let text = if slide.blocks.is_empty() { "Thanks!".to_string() } else { slide.text() };
    out.push(center(&dim(&text, opts), opts.width));
}

fn render_content(slide: &Slide, opts: &RenderOpts, out: &mut Vec<String>) {
    out.push(format!(" {}", bold(&slide.title(), opts)));
    out.push(String::new());
    for block in &slide.blocks {
        render_block(block, opts, out);
        if out.len() >= opts.height - 1 {
            break;
        }
    }
}

fn render_block(block: &Block, opts: &RenderOpts, out: &mut Vec<String>) {
    match block {
        Block::Heading { level, inlines } => {
            let prefix = "#".repeat(*level as usize);
            let text = inline_render(inlines, opts);
            out.push(format!(" {} {}", bold(&prefix, opts), text));
            out.push(String::new());
        }
        Block::Paragraph(inlines) => {
            out.push(format!(" {}", inline_render(inlines, opts)));
            out.push(String::new());
        }
        Block::List(items) | Block::OrderedList(items) => {
            for (i, item) in items.iter().enumerate() {
                let marker = if matches!(block, Block::OrderedList(_)) {
                    format!("{}. ", i + 1)
                } else {
                    "• ".to_string()
                };
                out.push(format!(" {}{}", marker, inline_render(item, opts)));
            }
            out.push(String::new());
        }
        Block::CodeBlock { code, .. } => {
            out.push(dim(" ┌─ code", opts));
            for line in code.lines() {
                out.push(dim(&format!(" │ {}", line), opts));
            }
            out.push(dim(" └─", opts));
            out.push(String::new());
        }
        Block::Quote(inlines) => {
            out.push(dim(&format!(" │ {}", inline_render(inlines, opts)), opts));
            out.push(String::new());
        }
        Block::Rule => {
            out.push("─".repeat(opts.width.saturating_sub(2)));
            out.push(String::new());
        }
        Block::Note(_) => {}
    }
}

fn footer(deck: &Deck, opts: &RenderOpts) -> String {
    let progress = deck.progress();
    let pct = (progress * 100.0).round() as u32;
    let pos = format!("{}/{}", deck.current + 1, deck.slides.len());
    let bar_width = 20usize;
    let filled = (progress * bar_width as f64).round() as usize;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));
    let text = format!(" {} {} {}%", bar, pos, pct);
    // Right-align.
    let pad = opts.width.saturating_sub(text.len());
    format!("{}{}", " ".repeat(pad), dim(&text, opts))
}

fn inline_render(inlines: &[Inline], opts: &RenderOpts) -> String {
    let mut s = String::new();
    for i in inlines {
        s.push_str(&inline_str(i, opts));
    }
    s
}

fn inline_str(i: &Inline, opts: &RenderOpts) -> String {
    if !opts.color {
        return inline_text(&[i.clone()]);
    }
    match i {
        Inline::Text(t) => t.clone(),
        Inline::Bold(t) => format!("{}{}{}", ansi::BOLD, t, ansi::RESET),
        Inline::Italic(t) => format!("{}{}{}", ansi::ITALIC, t, ansi::RESET),
        Inline::Code(t) => format!("{}`{}`{}", ansi::CYAN, t, ansi::RESET),
        Inline::Image { alt, .. } => format!("[img: {}]", alt),
        Inline::Link { text, url } => format!("{} ({})", text, url),
    }
}

fn inline_text(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for i in inlines {
        match i {
            Inline::Text(t) | Inline::Bold(t) | Inline::Italic(t) | Inline::Code(t) => s.push_str(t),
            Inline::Image { alt, .. } => s.push_str(&format!("[{}]", alt)),
            Inline::Link { text, .. } => s.push_str(text),
        }
    }
    s
}

fn center(text: &str, width: usize) -> String {
    // Strip ANSI for width calc.
    let plain: String = text.chars().filter(|c| !c.is_control()).collect();
    let pad = width.saturating_sub(plain.len()) / 2;
    format!("{}{}", " ".repeat(pad), text)
}

fn bold(s: &str, opts: &RenderOpts) -> String {
    if opts.color { format!("{}{}{}", ansi::BOLD, s, ansi::RESET) } else { s.to_string() }
}
fn dim(s: &str, opts: &RenderOpts) -> String {
    if opts.color { format!("{}{}{}", ansi::DIM, s, ansi::RESET) } else { s.to_string() }
}
fn cyan(s: &str, opts: &RenderOpts) -> String {
    if opts.color { format!("{}{}{}", ansi::CYAN, s, ansi::RESET) } else { s.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slide::SlideKind;

    fn slide() -> Slide {
        Slide {
            number: 1,
            blocks: vec![Block::Paragraph(vec![Inline::Text("Hello".into())])],
            notes: vec![],
            kind: SlideKind::Content,
        }
    }

    #[test]
    fn test_render_content() {
        let d = Deck::parse("# Title\n\nHello world\n\n---\n\n# Second\n\nMore text\n").unwrap();
        let s = &d.slides[1];
        let lines = render_slide(s, &d, &RenderOpts::default());
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("Second") || l.contains("More")));
    }

    #[test]
    fn test_render_title_centered() {
        let d = Deck::parse("# Big Title\n\nSubtitle here\n").unwrap();
        let lines = render_slide(&d.slides[0], &d, &RenderOpts::default());
        assert!(lines.iter().any(|l| l.contains("Big Title")));
    }

    #[test]
    fn test_plain_mode() {
        let d = Deck::parse("# T\n\n**bold** text\n").unwrap();
        let opts = RenderOpts { color: false, ..Default::default() };
        let lines = render_slide(&d.slides[0], &d, &opts);
        let all: String = lines.join("\n");
        assert!(!all.contains('\x1b'));
        assert!(all.contains("bold"));
    }
}
