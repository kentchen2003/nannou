pub use self::keys::Keys;
pub use self::mouse::Mouse;
pub use self::time::Time;
pub use self::window::Window;

/// Tracked state related to the focused window.
pub mod window {
    use geom;
    use window;
    use math::{BaseFloat, Vector2};

    /// The default scalar value used for window positioning and sizing.
    pub type DefaultScalar = geom::DefaultScalar;

    /// State of the window in focus.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Window<S = DefaultScalar> {
        /// ID of the window currently in focus.
        pub id: Option<window::Id>,
        /// The width of the focused window agnostic of DPI.
        ///
        /// This is equal to the pixel width divided by the hidpi_factor.
        pub width: S,
        /// The height of the focused window agnostic of DPI.
        ///
        /// This is equal to the pixel height divided by the hidpi_factor.
        pub height: S,
        /// The high "dots-per-inch" multiplier that describes the density of the screens pixels.
        pub hidpi_factor: S,
    }

    impl<S> Window<S>
    where
        S: BaseFloat,
    {
        /// Initialise the window state.
        pub fn new() -> Self {
            Window {
                id: None,
                width: S::zero(),
                height: S::zero(),
                hidpi_factor: S::one(),
            }
        }

        /// Get the range along the *x* axis occupied by the window.
        pub fn x_range(&self) -> geom::Range<S> {
            let half_w = self.width / (S::one() + S::one());
            geom::Range { start: -half_w, end: half_w }
        }

        /// Get the range along the *y* axis occupied by the window.
        pub fn y_range(&self) -> geom::Range<S> {
            let half_h = self.height / (S::one() + S::one());
            geom::Range { start: -half_h, end: half_h }
        }

        /// Get the x coordinate for the left edge of the window.
        pub fn rect(&self) -> geom::Rect<S> {
            let x = self.x_range();
            let y = self.y_range();
            geom::Rect { x, y }
        }

        /// Expects that there will be a `window::Id` (the common case) and **panic!**s otherwise.
        pub fn id(&self) -> window::Id {
            self.id.unwrap()
        }

        /// Return the `width` and `height` as a `Vector2`.
        pub fn size(&self) -> Vector2<S> {
            Vector2 { x: self.width, y: self.height }
        }
    }
}

/// Tracked state related to the keyboard.
pub mod keys {
    use event::{Key, ModifiersState};
    use std::collections::HashSet;
    use std::ops::Deref;

    /// The state of the keyboard.
    #[derive(Clone, Debug, Default)]
    pub struct Keys {
        /// The state of the modifier keys as last indicated by winit.
        pub mods: ModifiersState,
        /// The state of all keys as tracked via the nannou App event handling.
        pub down: Down,
    }

    /// The set of keys that are currently pressed.
    #[derive(Clone, Debug, Default)]
    pub struct Down {
        pub(crate) keys: HashSet<Key>,
    }

    impl Deref for Down {
        type Target = HashSet<Key>;
        fn deref(&self) -> &Self::Target {
            &self.keys
        }
    }
}

/// Tracked state related to the mouse.
pub mod mouse {
    use geom;
    use math::{BaseFloat, Point2};
    use std;
    use window;

    /// The default scalar value used for positions.
    pub type DefaultScalar = geom::DefaultScalar;

    #[doc(inline)]
    pub use event::MouseButton as Button;

    /// The max total number of buttons on a mouse.
    pub const NUM_BUTTONS: usize = 9;

    /// The state of the `Mouse` at a single moment in time.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Mouse<S = DefaultScalar> {
        /// The ID of the last window currently in focus.
        pub window: Option<window::Id>,
        /// *x* position relative to the middle of `window`.
        pub x: S,
        /// *y* position relative to the middle of `window`.
        pub y: S,
        /// A map describing the state of each mouse button.
        pub buttons: ButtonMap,
    }

    /// Whether the button is up or down.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ButtonPosition<S = DefaultScalar> {
        /// The button is up (i.e. pressed).
        Up,
        /// The button is down and was originally pressed down at the given `Point2`.
        Down(Point2<S>),
    }

    /// Stores the state of all mouse buttons.
    ///
    /// If the mouse button is down, it stores the position of the mouse when the button was pressed
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct ButtonMap<S = DefaultScalar> {
        buttons: [ButtonPosition<S>; NUM_BUTTONS],
    }

    /// An iterator yielding all pressed buttons.
    #[derive(Clone)]
    pub struct PressedButtons<'a, S: 'a = DefaultScalar> {
        buttons: std::iter::Enumerate<std::slice::Iter<'a, ButtonPosition<S>>>,
    }

    impl<S> Mouse<S>
    where
        S: BaseFloat,
    {
        /// Construct a new default `Mouse`.
        pub fn new() -> Self {
            Mouse {
                window: None,
                buttons: ButtonMap::new(),
                x: S::zero(),
                y: S::zero(),
            }
        }

        /// The position of the mouse relative to the middle of the window in focus..
        pub fn position(&self) -> Point2<S> {
            Point2 { x: self.x, y: self.y }
        }
    }

    impl<S> ButtonPosition<S>
    where
        S: BaseFloat,
    {
        /// If the mouse button is down, return a new one with position relative to the given `xy`.
        pub fn relative_to(self, xy: Point2<S>) -> Self {
            match self {
                ButtonPosition::Down(pos) => {
                    let rel_p = pos - xy;
                    ButtonPosition::Down(Point2 { x: rel_p.x, y: rel_p.y })
                }
                button_pos => button_pos,
            }
        }

        /// Is the `ButtonPosition` down.
        pub fn is_down(&self) -> bool {
            match *self {
                ButtonPosition::Down(_) => true,
                _ => false,
            }
        }

        /// Is the `ButtonPosition` up.
        pub fn is_up(&self) -> bool {
            match *self {
                ButtonPosition::Up => true,
                _ => false,
            }
        }

        /// Returns the position at which the button was pressed.
        pub fn if_down(&self) -> Option<Point2<S>> {
            match *self {
                ButtonPosition::Down(xy) => Some(xy),
                _ => None,
            }
        }
    }

    impl<S> ButtonMap<S>
    where
        S: BaseFloat,
    {
        /// Returns a new button map with all states set to `None`
        pub fn new() -> Self {
            ButtonMap { buttons: [ButtonPosition::Up; NUM_BUTTONS] }
        }

        /// Returns a copy of the ButtonMap relative to the given `Point`
        pub fn relative_to(self, xy: Point2<S>) -> Self {
            self.buttons.iter().enumerate().fold(
                ButtonMap::new(),
                |mut map,
                 (idx, button_pos)| {
                    map.buttons[idx] = button_pos.relative_to(xy);
                    map
                },
            )
        }

        /// The state of the left mouse button.
        pub fn left(&self) -> &ButtonPosition<S> {
            &self[Button::Left]
        }

        /// The state of the middle mouse button.
        pub fn middle(&self) -> &ButtonPosition<S> {
            &self[Button::Middle]
        }

        /// The state of the right mouse button.
        pub fn right(&self) -> &ButtonPosition<S> {
            &self[Button::Right]
        }

        /// Sets the `Button` in the `Down` position.
        pub fn press(&mut self, button: Button, xy: Point2<S>) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Down(xy);
        }

        /// Set's the `Button` in the `Up` position.
        pub fn release(&mut self, button: Button) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Up;
        }

        /// An iterator yielding all pressed mouse buttons along with the location at which they
        /// were originally pressed.
        pub fn pressed(&self) -> PressedButtons<S> {
            PressedButtons { buttons: self.buttons.iter().enumerate() }
        }
    }

    impl<S> std::ops::Index<Button> for ButtonMap<S> {
        type Output = ButtonPosition<S>;
        fn index(&self, button: Button) -> &Self::Output {
            &self.buttons[button_to_idx(button)]
        }
    }

    impl<'a, S> Iterator for PressedButtons<'a, S>
    where
        S: BaseFloat,
    {
        type Item = (Button, Point2<S>);
        fn next(&mut self) -> Option<Self::Item> {
            while let Some((idx, button_pos)) = self.buttons.next() {
                if let ButtonPosition::Down(xy) = *button_pos {
                    return Some((idx_to_button(idx), xy));
                }
            }
            None
        }
    }

    fn idx_to_button(i: usize) -> Button {
        match i {
            n @ 0...5 => Button::Other(n as u8),
            6 => Button::Left,
            7 => Button::Right,
            8 => Button::Middle,
            _ => Button::Other(std::u8::MAX),
        }
    }

    fn button_to_idx(button: Button) -> usize {
        match button {
            Button::Other(n) => n as usize,
            Button::Left => 6,
            Button::Right => 7,
            Button::Middle => 8,
        }
    }
}

/// Tracked durations related to the App.
pub mod time {
    use std::{ops, time};

    /// The state of time tracked by the App.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct Time {
        /// The duration since the app started running.
        pub since_start: Duration,
        /// The duration since the previous update.
        pub since_prev_update: Duration,
    }

    /// A wrapper around a std duration with simpler methods.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Duration {
        /// The inner std duration.
        pub duration: time::Duration,
    }

    impl From<time::Duration> for Duration {
        fn from(duration: time::Duration) -> Self {
            Duration { duration }
        }
    }

    impl Into<time::Duration> for Duration {
        fn into(self) -> time::Duration {
            self.duration
        }
    }

    impl ops::Deref for Duration {
        type Target = time::Duration;
        fn deref(&self) -> &Self::Target {
            &self.duration
        }
    }

    impl ops::DerefMut for Duration {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.duration
        }
    }

    impl Duration {
        /// A simple way of retrieving the duration as weeks.
        pub fn weeks(&self) -> f64 {
            self.days() / 7.0
        }

        /// A simple way of retrieving the duration as days.
        pub fn days(&self) -> f64 {
            self.hrs() / 24.0
        }

        /// A simple way of retrieving the duration as hrs.
        pub fn hrs(&self) -> f64 {
            self.mins() / 60.0
        }

        /// A simple way of retrieving the duration as minutes.
        pub fn mins(&self) -> f64 {
            self.secs() / 60.0
        }

        /// A simple way of retrieving the duration in seconds.
        pub fn secs(&self) -> f64 {
            self.as_secs() as f64 + self.subsec_nanos() as f64 * 1e-9
        }

        /// A simple way of retrieving the duration in milliseconds.
        pub fn ms(&self) -> f64 {
            self.secs() * 1_000.0
        }
    }
}