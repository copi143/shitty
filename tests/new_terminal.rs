#![cfg(feature = "font-unifont")]

#[test]
fn new_terminal() {
    let font = shitty::Unifont::new();
    let term = shitty::Terminal::new(80, 32, shitty::BufMode::None, vec![font]);
    drop(term);
}
