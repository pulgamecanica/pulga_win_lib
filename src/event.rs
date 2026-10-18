//! Our own event enum, decoupled from winit so the rest of the engine
//! (and its tests) never names a winit type directly.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Space,
    Escape,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Resized { width: u32, height: u32 },
    KeyDown(Key),
    KeyUp(Key),
    CloseRequested,
    RedrawRequested,
}

pub(crate) fn translate_key(code: winit::keyboard::KeyCode) -> Key {
    use winit::keyboard::KeyCode;
    match code {
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::Space => Key::Space,
        KeyCode::Escape => Key::Escape,
        _ => Key::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;

    #[test]
    fn translates_known_keys() {
        assert_eq!(translate_key(KeyCode::ArrowLeft), Key::Left);
        assert_eq!(translate_key(KeyCode::Space), Key::Space);
        assert_eq!(translate_key(KeyCode::Escape), Key::Escape);
    }

    #[test]
    fn unmapped_keys_fall_back_to_other() {
        assert_eq!(translate_key(KeyCode::KeyA), Key::Other);
    }
}
