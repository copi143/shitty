#![cfg(feature = "font-unifont")]

use std::sync::{Arc, Mutex};

use shitty::{BufMode, Terminal};

fn new_term(width: u32, height: u32) -> Terminal {
    Terminal::new(width, height, BufMode::Single, vec![shitty::Unifont::new()])
}

#[test]
fn new_initializes_grid_from_pixel_size() {
    let term = new_term(80, 32);
    assert_eq!(term.cols(), 10);
    assert_eq!(term.rows(), 2);
}

#[test]
fn resize_updates_grid_from_pixel_size() {
    let mut term = new_term(80, 32);
    term.resize(160, 48);
    assert_eq!(term.cols(), 20);
    assert_eq!(term.rows(), 3);
}

#[test]
#[should_panic(expected = "At least one font renderer must be provided")]
fn new_panics_without_renderer() {
    let _ = Terminal::new(80, 32, BufMode::Single, vec![]);
}

#[test]
fn input_forwards_to_pty_write_callback() {
    let term = new_term(80, 32);
    let sent = Arc::new(Mutex::new(Vec::<u8>::new()));
    let sent_clone = Arc::clone(&sent);

    term.callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
        sent_clone.lock().unwrap().extend_from_slice(s);
    }));

    term.user_input(b"hello");
    assert_eq!(&*sent.lock().unwrap(), b"hello");
}

#[test]
fn input_forwards_empty_and_utf8_text() {
    let term = new_term(80, 32);
    let sent = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let sent_clone = Arc::clone(&sent);

    term.callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
        sent_clone.lock().unwrap().push(s.to_vec());
    }));

    term.user_input(b"");
    term.user_input("中文🙂".as_bytes());

    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0], b"");
    assert_eq!(sent[1], "中文🙂".as_bytes());
}
