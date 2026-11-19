#![forbid(unsafe_code)]
//! hype - Markdown presentation engine for SigmaOS

#[derive(Debug, Clone)]
pub struct MarkdownSlide {
    pub raw_md: String,
    pub title: Option<String>,
    pub content_lines: Vec<String>,
}

impl MarkdownSlide {
    pub fn from_markdown(md: &str) -> Self {
        let mut title = None;
        let mut content_lines = Vec::new();
        for line in md.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") && title.is_none() {
                title = Some(trimmed[2..].to_string());
            } else if !trimmed.is_empty() {
                content_lines.push(line.to_string());
            }
        }
        Self {
            raw_md: md.to_string(),
            title,
            content_lines,
        }
    }
}

pub struct HypePresentation {
    slides: Vec<MarkdownSlide>,
    pub current: usize,
}

impl HypePresentation {
    pub fn from_markdown_string(md: &str) -> Self {
        let slides = md
            .split("\n---\n")
            .map(MarkdownSlide::from_markdown)
            .collect();
        Self { slides, current: 0 }
    }

    pub fn slide_count(&self) -> usize {
        self.slides.len()
    }

    pub fn current_slide(&self) -> Option<&MarkdownSlide> {
        self.slides.get(self.current)
    }

    pub fn next_slide(&mut self) -> bool {
        if self.current + 1 < self.slides.len() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    pub fn prev_slide(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod hype_extra_tests {
    use super::*;

    #[test]
    fn test_slide_boundary_navigation_aaa() {
        // Arrange
        let md = "# Only One Slide";
        let mut pres = HypePresentation::from_markdown_string(md);
        // Act & Assert
        assert!(!pres.prev_slide());
        assert!(!pres.next_slide());
        assert_eq!(pres.slide_count(), 1);
    }

    #[test]
    fn test_empty_markdown_presentation_aaa() {
        // Arrange
        let pres = HypePresentation::from_markdown_string("");
        // Act & Assert
        assert_eq!(pres.slide_count(), 1);
        assert_eq!(pres.current_slide().unwrap().title, None);
    }

    #[test]
    fn test_markdown_slide_content_lines_aaa() {
        // Arrange
        let raw = "# Title\nLine 1\nLine 2";
        // Act
        let slide = MarkdownSlide::from_markdown(raw);
        // Assert
        assert_eq!(slide.title.as_deref(), Some("Title"));
        assert_eq!(slide.content_lines.len(), 2);
    }
}
