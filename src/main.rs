use sigmaos_tools_hype::HypePresentation;

fn main() {
    let md = "# Welcome\nFirst slide\n---\n# Next\nSecond slide";
    let pres = HypePresentation::from_markdown_string(md);
    println!("Loaded presentation with {} slides", pres.slide_count());
}
