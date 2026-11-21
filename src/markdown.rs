//! Minimal markdown block/inline parser for slides.
//!
//! Supports the subset hype decks use: ATX headers, unordered/ordered
//! lists, fenced code blocks, images, bold/italic/code inline, and
//! `---` horizontal rules (slide separators handled at deck level).

/// An inline text element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(String),
    /// Bold `**text**`.
    Bold(String),
    /// Italic `*text*`.
    Italic(String),
    /// Inline code `` `code` ``.
    Code(String),
    /// Image `![alt](src)`.
    Image { alt: String, src: String },
    /// Link `[text](url)`.
    Link { text: String, url: String },
}

/// A markdown block element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Heading level 1-6.
    Heading { level: u8, inlines: Vec<Inline> },
    /// Paragraph.
    Paragraph(Vec<Inline>),
    /// Unordered list items.
    List(Vec<Vec<Inline>>),
    /// Ordered list items.
    OrderedList(Vec<Vec<Inline>>),
    /// Fenced code block.
    CodeBlock { lang: String, code: String },
    /// Blockquote.
    Quote(Vec<Inline>),
    /// Horizontal rule.
    Rule,
    /// Presenter note (`???` or `<!-- note -->`).
    Note(Vec<Inline>),
}

/// Parse inline markdown (`**`, `*`, `` ` ``, `![]()`, `[]()`).
pub fn parse_inline(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        // Image ![alt](src)
        if let Some(start) = rest.find("![") {
            if start > 0 {
                out.extend(parse_inline(&rest[..start]));
            }
            let after = &rest[start + 2..];
            if let Some(close) = after.find("](") {
                let alt = &after[..close];
                let url_part = &after[close + 2..];
                if let Some(end) = url_part.find(')') {
                    out.push(Inline::Image {
                        alt: alt.to_string(),
                        src: url_part[..end].to_string(),
                    });
                    rest = &url_part[end + 1..];
                    continue;
                }
            }
            out.push(Inline::Text(rest[start..].to_string()));
            break;
        }
        // Inline code
        if let Some(start) = rest.find('`') {
            if start > 0 {
                out.extend(parse_inline(&rest[..start]));
            }
            let after = &rest[start + 1..];
            if let Some(end) = after.find('`') {
                out.push(Inline::Code(after[..end].to_string()));
                rest = &after[end + 1..];
                continue;
            }
            out.push(Inline::Text(rest[start..].to_string()));
            break;
        }
        // Bold **
        if let Some(start) = rest.find("**") {
            if start > 0 {
                out.extend(parse_inline(&rest[..start]));
            }
            let after = &rest[start + 2..];
            if let Some(end) = after.find("**") {
                out.push(Inline::Bold(after[..end].to_string()));
                rest = &after[end + 2..];
                continue;
            }
            out.push(Inline::Text(rest[start..].to_string()));
            break;
        }
        // Italic *
        if let Some(start) = rest.find('*') {
            if start > 0 {
                out.extend(parse_inline(&rest[..start]));
            }
            let after = &rest[start + 1..];
            if let Some(end) = after.find('*') {
                out.push(Inline::Italic(after[..end].to_string()));
                rest = &after[end + 1..];
                continue;
            }
            out.push(Inline::Text(rest[start..].to_string()));
            break;
        }
        // Link [text](url)
        if let Some(start) = rest.find('[') {
            if start > 0 {
                out.extend(parse_inline(&rest[..start]));
            }
            let after = &rest[start + 1..];
            if let Some(close) = after.find("](") {
                let text_part = &after[..close];
                let url_part = &after[close + 2..];
                if let Some(end) = url_part.find(')') {
                    out.push(Inline::Link {
                        text: text_part.to_string(),
                        url: url_part[..end].to_string(),
                    });
                    rest = &url_part[end + 1..];
                    continue;
                }
            }
            out.push(Inline::Text(rest[start..].to_string()));
            break;
        }
        out.push(Inline::Text(rest.to_string()));
        break;
    }
    out
}

/// Parse a full slide's markdown into blocks.
pub fn parse_markdown(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        // Fenced code block ```
        if trimmed.starts_with("```") {
            let lang = trimmed[3..].trim().to_string();
            let mut code = String::new();
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code.push_str(lines[i]);
                code.push('\n');
                i += 1;
            }
            i += 1; // skip closing ```
            blocks.push(Block::CodeBlock { lang, code });
            continue;
        }
        // Heading #..######
        if let Some(level) = heading_level(trimmed) {
            blocks.push(Block::Heading {
                level,
                inlines: parse_inline(trimmed[level as usize..].trim()),
            });
            i += 1;
            continue;
        }
        // Horizontal rule ---
        if trimmed == "---" || trimmed == "***" {
            blocks.push(Block::Rule);
            i += 1;
            continue;
        }
        // Presenter note ??? or <!-- note -->
        if trimmed.starts_with("???") {
            let note_text = trimmed[3..].trim();
            blocks.push(Block::Note(parse_inline(note_text)));
            i += 1;
            continue;
        }
        // Blockquote >
        if trimmed.starts_with('>') {
            blocks.push(Block::Quote(parse_inline(trimmed[1..].trim())));
            i += 1;
            continue;
        }
        // List items - or *
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let mut items = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim();
                if l.starts_with("- ") || l.starts_with("* ") {
                    items.push(parse_inline(&l[2..]));
                    i += 1;
                } else {
                    break;
                }
            }
            blocks.push(Block::List(items));
            continue;
        }
        // Ordered list 1.
        if is_ordered_item(trimmed) {
            let mut items = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim();
                if is_ordered_item(l) {
                    let content = l.split_once('.').map(|(_, c)| c.trim()).unwrap_or(l);
                    items.push(parse_inline(content));
                    i += 1;
                } else {
                    break;
                }
            }
            blocks.push(Block::OrderedList(items));
            continue;
        }
        // Paragraph — gather until blank line or block start.
        let mut para = String::new();
        while i < lines.len() {
            let l = lines[i];
            let t = l.trim();
            if t.is_empty()
                || heading_level(t).is_some()
                || t.starts_with("```")
                || t == "---"
                || t.starts_with("> ")
                || t.starts_with("- ")
                || is_ordered_item(t)
            {
                break;
            }
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(t);
            i += 1;
        }
        if !para.is_empty() {
            blocks.push(Block::Paragraph(parse_inline(&para)));
        }
    }
    blocks
}

fn heading_level(line: &str) -> Option<u8> {
    let level = line.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&level) && line[level..].starts_with(' ') {
        Some(level as u8)
    } else {
        None
    }
}

fn is_ordered_item(line: &str) -> bool {
    // "1. text" or "12) text"
    let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
    !digits.is_empty()
        && line[digits.len()..].starts_with(". ")
        || line[digits.len()..].starts_with(") ")
}
