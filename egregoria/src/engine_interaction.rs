use geom::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Movable;

empty_inspect_impl!(Movable);

#[derive(Clone, Serialize, Deserialize)]
pub struct Selectable {
    pub radius: f32,
}

impl Selectable {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Default for Selectable {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Default)]
pub struct RenderStats {
    pub(crate) update_time: [f32; 10], // over the last 10 frames
    pub(crate) render_time: [f32; 10],
}

impl RenderStats {
    pub fn add_update_time(&mut self, update_time: f32) {
        self.update_time.as_mut().rotate_right(1);
        self.update_time[0] = update_time;
    }

    pub fn add_render_time(&mut self, render_time: f32) {
        self.render_time.as_mut().rotate_right(1);
        self.render_time[0] = render_time;
    }

    pub fn update_time_avg(&self) -> f32 {
        (&self.update_time).iter().sum::<f32>() / (self.update_time.len() as f32)
    }

    pub fn render_time_avg(&self) -> f32 {
        (&self.render_time).iter().sum::<f32>() / (self.render_time.len() as f32)
    }
}

#[derive(Clone, Copy)]
pub struct TimeInfo {
    pub delta: f32,
    pub time: f64,
    pub time_seconds: u64,
    pub time_speed: f32,
}

impl Default for TimeInfo {
    fn default() -> Self {
        Self {
            delta: 0.0,
            time: 0.0,
            time_seconds: 0,
            time_speed: 1.0,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

#[derive(Clone)]
pub struct MouseInfo {
    pub wheel_delta: f32,
    pub screen: Vec2,
    pub unprojected: Vec2,
    pub buttons: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
}

impl Default for MouseInfo {
    fn default() -> Self {
        MouseInfo {
            wheel_delta: 0.0,
            screen: Vec2::ZERO,
            unprojected: Vec2::ZERO,
            buttons: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }
}

#[derive(Clone)]
pub struct KeyboardInfo {
    pub just_pressed: HashSet<KeyCode>,
    pub is_pressed: HashSet<KeyCode>,
    pub last_characters: Vec<char>,
}

impl Default for KeyboardInfo {
    fn default() -> Self {
        KeyboardInfo {
            just_pressed: HashSet::new(),
            is_pressed: HashSet::new(),
            last_characters: Vec::with_capacity(4),
        }
    }
}

/// Symbolic name for a keyboard key.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}
