use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn is_char_event(k: &KeyEvent, c: char) -> bool {
    k.code == KeyCode::Char(c)
}

pub fn is_ctrl_alt_char_event(k: &KeyEvent, c: char) -> bool {
    k.code == KeyCode::Char(c)
        && k.modifiers.contains(KeyModifiers::CONTROL)
        && k.modifiers.contains(KeyModifiers::ALT)
}

pub fn is_shift_char_event(k: &KeyEvent, c: char) -> bool {
    (k.code == KeyCode::Char(c) || k.code == KeyCode::Char(c.to_ascii_uppercase()))
        && k.modifiers.contains(KeyModifiers::SHIFT)
}
