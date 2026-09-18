use std::sync::{Arc, Mutex};

use shitty::{BufMode, EmptyFontRenderer, Event, PointerButton, Terminal};

fn new_term() -> Terminal {
    Terminal::new(80, 32, BufMode::Single, vec![EmptyFontRenderer::new(8, 16)])
}

fn capture(term: &Terminal) -> Arc<Mutex<Vec<Vec<u8>>>> {
    let sent = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let sent_clone = Arc::clone(&sent);
    term.callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
        sent_clone.lock().unwrap().push(s.to_vec());
    }));
    sent
}

#[test]
fn encoding_without_tracking_does_not_report() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1006h");
    term.handle_event(Event::PointerPress(PointerButton::Left));
    assert!(sent.lock().unwrap().is_empty());
}

#[test]
fn sgr_click_tracking() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1000;1006h");
    term.handle_event(Event::PointerMove(0, 0));
    term.handle_event(Event::PointerPress(PointerButton::Left));
    term.handle_event(Event::PointerRelease(PointerButton::Left));
    term.process(b"\x1b[?1000;1006l");
    term.handle_event(Event::PointerPress(PointerButton::Left));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[<0;1;1M".to_vec(), b"\x1b[<0;1;1m".to_vec()]);
}

#[test]
fn cell_motion_only_when_cell_changes() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1002;1006h");
    term.handle_event(Event::PointerPress(PointerButton::Left));
    term.handle_event(Event::PointerMove(1, 0));
    term.handle_event(Event::PointerMove(8, 0));
    term.handle_event(Event::PointerMove(9, 0));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[<0;1;1M".to_vec(), b"\x1b[<32;2;1M".to_vec()]);
}

#[test]
fn wheel_is_reported_instead_of_history_scroll() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1000;1006h");
    assert!(!term.handle_event(Event::scroll(1)));
    assert!(!term.handle_event(Event::scroll(-2)));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[<64;1;1M".to_vec(), b"\x1b[<65;1;1M".to_vec(), b"\x1b[<65;1;1M".to_vec(),]);
}

#[test]
fn alternate_scroll_sends_cursor_keys() {
    let mut term = new_term();
    let sent = capture(&term);
    term.set_scroll_speed(1);
    term.process(b"\x1b[?1049h\x1b[?1007h");
    assert!(!term.handle_event(Event::scroll(1)));
    assert!(!term.handle_event(Event::scroll(-1)));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[A".to_vec(), b"\x1b[B".to_vec()]);
}

#[test]
fn focus_events_require_mode_1004() {
    let mut term = new_term();
    let sent = capture(&term);
    term.handle_event(Event::Focus(true));
    term.process(b"\x1b[?1004h");
    term.handle_event(Event::Focus(true));
    term.handle_event(Event::Focus(false));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[I".to_vec(), b"\x1b[O".to_vec()]);
}

#[test]
fn reset_clears_mouse_modes() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1000;1006h");
    term.reset();
    term.handle_event(Event::PointerPress(PointerButton::Left));
    assert!(sent.lock().unwrap().is_empty());
}

#[test]
fn leave_does_not_release() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1002;1006h");
    term.handle_event(Event::PointerPress(PointerButton::Left));
    term.handle_event(Event::PointerLeave);
    term.handle_event(Event::PointerMove(-8, 0));
    term.handle_event(Event::PointerRelease(PointerButton::Left));
    let sent = sent.lock().unwrap();
    assert_eq!(
        sent.as_slice(),
        [b"\x1b[<0;1;1M".to_vec(), b"\x1b[<32;1;1M".to_vec(), b"\x1b[<0;1;1m".to_vec()]
    );
}

#[test]
fn outside_motion_is_clamped() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1003;1006h");
    term.handle_event(Event::PointerMove(10000, 10000));
    assert_eq!(sent.lock().unwrap().as_slice(), [b"\x1b[<35;10;2M".to_vec()]);
}

#[test]
fn sgr_pixels_uses_pixel_coordinates() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1000;1016h");
    term.handle_event(Event::PointerMove(16, 8));
    term.handle_event(Event::PointerPress(PointerButton::Left));
    assert_eq!(sent.lock().unwrap().as_slice(), [b"\x1b[<0;17;9M".to_vec()]);
}

#[test]
fn decrqm_reports_mouse_mode() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1000$p");
    term.process(b"\x1b[?1000h");
    term.process(b"\x1b[?1000$p");
    let sent = sent.lock().unwrap();
    assert_eq!(sent.as_slice(), [b"\x1b[?1000;2$y".to_vec(), b"\x1b[?1000;1$y".to_vec()]);
}

#[test]
fn highlight_tracking_reports_clicks() {
    let mut term = new_term();
    let sent = capture(&term);
    term.process(b"\x1b[?1001;1006h");
    term.handle_event(Event::PointerPress(PointerButton::Left));
    assert_eq!(sent.lock().unwrap().as_slice(), [b"\x1b[<0;1;1M".to_vec()]);
}
