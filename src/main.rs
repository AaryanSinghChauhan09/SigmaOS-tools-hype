//! hype — Markdown presentation tool (CLI front-end over the deck model).

use sigmaos_tools_hype::Deck;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned();
    let deck = if let Some(p) = path {
        match Deck::load(&p) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("hype: {e}");
                std::process::exit(1);
            }
        }
    } else {
        // Demo: parse a tiny built-in deck.
        Deck::parse("# Welcome\n\nFirst slide\n\n---\n\n# Next\n\nSecond slide\n").unwrap()
    };
    println!("hype — {} slides", deck.len());
    for s in &deck.slides {
        println!("  {}. {} ({:?})", s.number, s.title(), s.kind);
    }
}
