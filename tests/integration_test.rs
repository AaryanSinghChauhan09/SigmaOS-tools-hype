use sigmaos_tools_hype::HypePresentation;

#[test]
fn test_parse_slides_aaa() {
    // Arrange
    let md = "# Slide 1\nContent 1\n---\n# Slide 2\nContent 2";
    // Act
    let mut pres = HypePresentation::from_markdown_string(md);
    // Assert
    assert_eq!(pres.slide_count(), 2);
    assert_eq!(
        pres.current_slide().unwrap().title.as_deref(),
        Some("Slide 1")
    );
    assert!(pres.next_slide());
    assert_eq!(
        pres.current_slide().unwrap().title.as_deref(),
        Some("Slide 2")
    );
}
