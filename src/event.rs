//! Our own event enum, decoupled from winit so the rest of the engine
//! (and its tests) never names a winit type directly.

/// A keyboard key, by physical position (a US-layout name), so WASD stays
/// under the same fingers on any keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Space bar.
    Space,
    /// Escape.
    Escape,
    /// Enter / Return.
    Enter,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Tab.
    Tab,
    /// Home.
    Home,
    /// End.
    End,
    /// Either Shift key.
    Shift,
    /// Either Control key (Command is not mapped here).
    Control,
    /// A letter `'a'..='z'` (always lowercase) or a digit `'0'..='9'`.
    Char(char),
    /// Function key F1..F12, by number.
    F(u8),
    /// Anything not listed above.
    Other,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Primary (usually left) button.
    Left,
    /// Secondary (usually right) button.
    Right,
    /// Wheel click.
    Middle,
}

/// Something that happened to the window, as delivered to [`crate::App::event`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// The drawable area changed size, in physical pixels.
    Resized {
        /// New width in pixels.
        width: u32,
        /// New height in pixels.
        height: u32,
    },
    /// A key went down (repeats while held, at the OS key-repeat rate).
    KeyDown(Key),
    /// A key went up.
    KeyUp(Key),
    /// A character was typed, after keyboard layout, Shift and dead keys
    /// (for text fields). Sent after the matching [`Event::KeyDown`];
    /// control characters are not sent.
    Text(char),
    /// The cursor moved, in physical pixels from the window's top-left.
    MouseMoved {
        /// Pixels from the left edge.
        x: f32,
        /// Pixels from the top edge.
        y: f32,
    },
    /// A mouse button went down at the last [`Event::MouseMoved`] position.
    MouseDown(MouseButton),
    /// A mouse button went up at the last [`Event::MouseMoved`] position.
    MouseUp(MouseButton),
    /// Wheel scrolled; positive is away from the user ("up"), in lines.
    Wheel(f32),
    /// The user asked to close the window.
    CloseRequested,
    /// A frame is about to be drawn.
    RedrawRequested,
}

pub(crate) fn translate_key(code: winit::keyboard::KeyCode) -> Key {
    use winit::keyboard::KeyCode as K;
    const LETTERS: [K; 26] = [
        K::KeyA,
        K::KeyB,
        K::KeyC,
        K::KeyD,
        K::KeyE,
        K::KeyF,
        K::KeyG,
        K::KeyH,
        K::KeyI,
        K::KeyJ,
        K::KeyK,
        K::KeyL,
        K::KeyM,
        K::KeyN,
        K::KeyO,
        K::KeyP,
        K::KeyQ,
        K::KeyR,
        K::KeyS,
        K::KeyT,
        K::KeyU,
        K::KeyV,
        K::KeyW,
        K::KeyX,
        K::KeyY,
        K::KeyZ,
    ];
    const DIGITS: [K; 10] = [
        K::Digit0,
        K::Digit1,
        K::Digit2,
        K::Digit3,
        K::Digit4,
        K::Digit5,
        K::Digit6,
        K::Digit7,
        K::Digit8,
        K::Digit9,
    ];
    const FUNCTION: [K; 12] = [
        K::F1,
        K::F2,
        K::F3,
        K::F4,
        K::F5,
        K::F6,
        K::F7,
        K::F8,
        K::F9,
        K::F10,
        K::F11,
        K::F12,
    ];
    if let Some(i) = LETTERS.iter().position(|&k| k == code) {
        return Key::Char((b'a' + i as u8) as char);
    }
    if let Some(i) = DIGITS.iter().position(|&k| k == code) {
        return Key::Char((b'0' + i as u8) as char);
    }
    if let Some(i) = FUNCTION.iter().position(|&k| k == code) {
        return Key::F(i as u8 + 1);
    }
    match code {
        K::ArrowLeft => Key::Left,
        K::ArrowRight => Key::Right,
        K::ArrowUp => Key::Up,
        K::ArrowDown => Key::Down,
        K::Space => Key::Space,
        K::Escape => Key::Escape,
        K::Enter | K::NumpadEnter => Key::Enter,
        K::Backspace => Key::Backspace,
        K::Delete => Key::Delete,
        K::Tab => Key::Tab,
        K::Home => Key::Home,
        K::End => Key::End,
        K::ShiftLeft | K::ShiftRight => Key::Shift,
        K::ControlLeft | K::ControlRight => Key::Control,
        _ => Key::Other,
    }
}

pub(crate) fn translate_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
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
        assert_eq!(translate_key(KeyCode::ShiftRight), Key::Shift);
    }

    #[test]
    fn letters_digits_and_function_keys() {
        assert_eq!(translate_key(KeyCode::KeyA), Key::Char('a'));
        assert_eq!(translate_key(KeyCode::KeyZ), Key::Char('z'));
        assert_eq!(translate_key(KeyCode::Digit7), Key::Char('7'));
        assert_eq!(translate_key(KeyCode::F5), Key::F(5));
    }

    #[test]
    fn unmapped_keys_fall_back_to_other() {
        assert_eq!(translate_key(KeyCode::CapsLock), Key::Other);
    }

    #[test]
    fn extra_mouse_buttons_are_dropped() {
        assert_eq!(
            translate_button(winit::event::MouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(translate_button(winit::event::MouseButton::Back), None);
    }
}
