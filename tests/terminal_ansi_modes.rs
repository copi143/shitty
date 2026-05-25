#![cfg(all(feature = "font-unifont", feature = "vte"))]

use std::sync::{Arc, Mutex};

use shitty::{BufMode, Terminal};

fn new_term() -> Terminal {
    Terminal::new(80, 32, BufMode::Single, vec![shitty::Unifont::new()])
}

#[test]
fn bracketed_paste_mode_toggles_output_sequence() {
    let mut term = new_term();
    let sent = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let sent_clone = Arc::clone(&sent);

    term.callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
        sent_clone.lock().unwrap().push(s.to_vec());
    }));

    term.paste("plain");
    term.process(b"\x1b[?2004h");
    term.paste("wrapped");
    term.process(b"\x1b[?2004l");
    term.paste("plain-again");

    let sent = sent.lock().unwrap();
    assert_eq!(sent[0], b"plain");
    assert_eq!(sent[1], b"\x1b[200~wrapped\x1b[201~");
    assert_eq!(sent[2], b"plain-again");
}

#[test]
fn bracketed_paste_repeated_toggles_keep_consistent_state() {
    let mut term = new_term();
    let sent = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let sent_clone = Arc::clone(&sent);

    term.callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
        sent_clone.lock().unwrap().push(s.to_vec());
    }));

    term.process(b"\x1b[?2004h");
    term.paste("a");
    term.process(b"\x1b[?2004l");
    term.paste("b");

    let sent = sent.lock().unwrap();
    assert_eq!(sent[0], b"\x1b[200~a\x1b[201~");
    assert_eq!(sent[1], b"b");
}
