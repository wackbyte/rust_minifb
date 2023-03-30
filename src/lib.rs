//! minifb is a cross platform library written in [Rust](https://www.rust-lang.org) that makes it
//! easy to open windows (usually native to the running operating system) and can optionally show
//! a 32-bit buffer. minifb also support keyboard, mouse input and menus on selected operating
//! systems.
#![deny(missing_debug_implementations)]

#[cfg(not(any(target_os = "macos", target_os = "redox", windows)))]
#[cfg(feature = "wayland")]
#[macro_use]
extern crate dlib;

mod buffer_helper;
mod error;
mod icon;
mod key;
mod key_handler;
mod mouse_handler;
mod os;
mod rate;
mod window_flags;

pub use self::error::{Error, Result};
pub use self::icon::Icon;
pub use self::key::Key;
pub use raw_window_handle::HasRawWindowHandle;

use std::fmt;
use std::os::raw;

use bitflags::bitflags;

/// The pixel scale of the framebuffer.
///
/// For example, this could be used to display a 320x256 buffer at a large size on a screen with a much higher resolution.
#[derive(Clone, Copy, Debug)]
pub enum Scale {
    /// Checks your current screen resolution and calculates the largest window size that can be used within that limit.
    /// Useful if you have a small buffer to display on a high-resolution screen.
    FitScreen,
    /// 1x scale.
    ///
    /// Does not do any scaling.
    X1,
    /// 2x scale.
    ///
    /// Example: 320x200 -> 640x400
    X2,
    /// 4x scale.
    ///
    /// Example: 320x200 -> 1280x800
    X4,
    /// 8x scale.
    ///
    /// Example: 320x200 -> 2560x1600
    X8,
    /// 16x scale.
    ///
    /// Example: 320x200 -> 5120x3200
    X16,
    /// 32x scale.
    ///
    /// Example: 320x200 -> 10240x6400
    X32,
}

/// Defines how the buffer should be displayed if the window is resized.
///
/// On some platforms (such as X11), it's possible for a window to be resized even if it's explicitly set as non-resizable.
/// This could cause issues with how the buffer is displayed.
/// Thankfully, you may specify a behavior to use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaleMode {
    /// Stretches the buffer to fit the entire window.
    ///
    /// For example, if your buffer is 256x256 and your window is 1024x1024, it will be scaled up 4 times.
    Stretch,
    /// Keeps the correct aspect ratio while fully scaling up on one axis.
    ///
    /// The borders will be filled with the window's background color.
    AspectRatioStretch,
    /// Places the buffer in the middle of the window without any scaling.
    ///
    /// The borders will be filled with the window's background color.
    ///
    /// If the window is smaller than the buffer, the center of the buffer will be displayed.
    Center,
    /// Same as [`Center`](Self::Center), but places the buffer in the upper-left corner of the window.
    UpperLeft,
}

/// Indicates whether key repeat should be used or not.
///
/// Used in [`Window::is_key_pressed`] and [`Window::get_keys_pressed`] to decide if keypresses from key repeat should be counted.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum KeyRepeat {
    /// Use key repeat.
    Yes,
    /// Don't use key repeat.
    No,
}

/// The various mouse buttons that are available.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Middle mouse button.
    Middle,
    /// Right mouse button.
    Right,
}

/// Different modes for deciding how mouse coordinates should be handled.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MouseMode {
    /// Returns the mouse coordinates even if they are outside the window.
    /// Note that they may be negative.
    Pass,
    /// Clamps the mouse coordinates to within the bounds of the window.
    Clamp,
    /// Discards the mouse coordinates if they are outside the window.
    Discard,
}

/// Different styles of cursor that can be used.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CursorStyle {
    /// A pointer. This is what the cursor normally looks like.
    Arrow,
    /// An I-beam, useful for indicating insertion (like a text field).
    Ibeam,
    /// A cross-hair.
    Crosshair,
    /// A closed hand, useful for indicating that something is being dragged.
    /// May use the default hand on unsupported platforms.
    ClosedHand,
    /// An open hand, useful for indicating that something is draggable.
    /// May use the default hand on unsupported platforms.
    OpenHand,
    /// Indicates resizing in the horizontal (left-right) direction.
    ResizeLeftRight,
    /// Indicates resizing in the vertical (up-down) direction.
    ResizeUpDown,
    /// Indicates resizing in all directions.
    ResizeAll,
}

/// A callback to use when inputs are received.
///
/// To use a callback, add it to the window with [`Window::set_input_callback`].
pub trait InputCallback {
    /// Called when text is added to the window.
    ///
    /// This only passes in a unicode character.
    /// Modifiers such as [`Key::LeftShift`] are **not** reported.
    fn add_char(&mut self, uni_char: u32);

    /// Called whenever a key is pressed or released.
    ///
    /// This reports the state of the key (`true` is pressed, `false` is released).
    /// Modifiers such as [`Key::LeftShift`] are included.
    fn set_key_state(&mut self, _key: Key, _state: bool) {}
}

/// Creation settings for a [`Window`].
///
/// By default, the settings are optimal for displaying a 32-bit buffer (for example: resizing is disabled).
#[derive(Clone, Copy, Debug)]
pub struct WindowOptions {
    /// Whether or not the window should be borderless. (default: `false`)
    pub borderless: bool,
    /// Whether or not the window should have a title. (default: `true`)
    pub title: bool,
    /// Whether or not it should be possible to resize the window. (default: `false`)
    pub resize: bool,
    /// Adjusts the pixel scale of the buffer used with [`Window::update_with_buffer`]. (default: [`X1`](Scale::X1))
    pub scale: Scale,
    /// Defines how the buffer used with [`Window::update_with_buffer`] should be stretched if the window is resized. (default: [`Stretch`](ScaleMode::Stretch))
    pub scale_mode: ScaleMode,
    /// Whether or not the window should be the topmost window. (default: `false`)
    pub topmost: bool,
    /// Whether or not the window is allowed to draw transparent pixels. (default: `false`)
    ///
    /// Requires `borderless` to be `true`.
    ///
    /// # Platform-specific behavior
    ///
    /// Requires `none` to be `true` on Windows.
    ///
    /// Currently unimplemented on macOS.
    // TODO: Currently not implemented on OSX.
    // TODO: Make it work without none option on windows.
    pub transparency: bool,
    /// Required for transparency on Windows. (default: `false`)
    ///
    /// Should be mutually exclusive to `resize`, automatically assumes `borderless`.
    ///
    /// # Platform-specific behavior
    ///
    /// Not supported on macOS.
    pub none: bool,
}

impl Default for WindowOptions {
    fn default() -> WindowOptions {
        WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X1,
            scale_mode: ScaleMode::Stretch,
            topmost: false,
            transparency: false,
            none: false,
        }
    }
}

#[cfg(target_os = "macos")]
use self::os::macos as imp;
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use self::os::posix as imp;
#[cfg(target_os = "redox")]
use self::os::redox as imp;
#[cfg(target_arch = "wasm32")]
use self::os::wasm as imp;
#[cfg(target_os = "windows")]
use self::os::windows as imp;

/// A window. May be used to display a 32-bit framebuffer.
pub struct Window(imp::Window);

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Window").field(&format_args!("..")).finish()
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.0.raw_window_handle()
    }
}

impl Window {
    /// Opens up a new window.
    ///
    /// # Examples
    ///
    /// Open up a window with the default settings:
    ///
    /// ```no_run
    /// # use minifb::*;
    /// let mut window = match Window::new("Test", 640, 400, WindowOptions::default()) {
    ///     Ok(win) => win,
    ///     Err(err) => {
    ///         println!("Unable to create window: {}", err);
    ///         return;
    ///     }
    /// };
    /// ```
    ///
    /// Open up a window that is resizable:
    ///
    /// ```no_run
    /// # use minifb::*;
    /// let mut window = Window::new(
    ///     "Test",
    ///     640,
    ///     400,
    ///     WindowOptions {
    ///         resize: true,
    ///         ..Default::default()
    ///     },
    /// )
    /// .expect("Unable to create window");
    /// ```
    pub fn new(name: &str, width: usize, height: usize, opts: WindowOptions) -> Result<Window> {
        if opts.transparency && !opts.borderless {
            return Err(Error::WindowCreate(
                "Window transparency requires the borderless property".to_owned(),
            ));
        }
        imp::Window::new(name, width, height, opts).map(Window)
    }

    /// Sets the title of the window after creation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    ///
    /// window.set_title("My New Title!");
    /// ```
    pub fn set_title(&mut self, title: &str) {
        self.0.set_title(title)
    }

    /// Sets the icon of the window after creation.
    ///
    /// # Platform-specific behavior
    ///
    /// The type of data within the icon depends on the current platform:
    ///
    /// - **Windows**:
    ///   A path to a `.ico` file relative the current working directory.
    ///   To set the icon of the executable, see the `rc.exe` tool.
    /// - **macOS**:
    ///   (not implemented)
    /// - **X11**:
    ///   A `u64` buffer of ARGB data.
    /// - **Wayland**:
    ///   (not implemented)
    /// - **Web**:
    ///   (not implemented)
    /// - **RedoxOS**:
    ///   (not implemented)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # use std::str::FromStr;
    /// let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    ///
    /// #[cfg(target_os = "windows")]
    /// window.set_icon(Icon::from_str("src/icon.ico").unwrap());
    /// ```
    pub fn set_icon(&mut self, icon: Icon) {
        self.0.set_icon(icon)
    }

    /// Returns an opaque pointer to the native window handle.
    ///
    /// # Platform-specific behavior
    ///
    /// The type of the handle depends on the current platform:
    ///
    /// - **Windows**:
    ///   `HWND`
    /// - **macOS**:
    ///   `NSWindow`
    /// - **X11**:
    ///   `XWindow`
    /// - **Wayland**:
    ///   `WlSurface`
    /// - **Web**:
    ///   (none)
    /// - **RedoxOS**:
    ///   (none)
    #[inline]
    pub fn get_window_handle(&self) -> *mut raw::c_void {
        self.0.get_window_handle()
    }

    /// Updates the window and draws a 32-bit framebuffer. Receives any new keyboard or mouse input.
    ///
    /// The encoding for each pixel is `ARGB`:
    ///
    /// - the upper 8 bits (25-32) are for the alpha channel,
    /// - the next 8 bits (17-24) are for the red channel,
    /// - the next 8 bits (9-16) are for the green channel,
    /// - and the lower 8-bits (1-8) are for the blue channel.
    ///
    /// The buffer should be at least the size of the window.
    ///
    /// **Notice:** Only **one** of this function or [`update`](Self::update) should be used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ///     let (r, g, b) = (r as u32, g as u32, b as u32);
    ///     (r << 16) | (g << 8) | b
    /// }
    ///
    /// let window_width = 600;
    /// let window_height = 400;
    /// let buffer_width = 600;
    /// let buffer_height = 400;
    ///
    /// let azure_blue = rgb(0, 127, 255);
    ///
    /// let mut buffer: Vec<u32> = vec![azure_blue; buffer_width * buffer_height];
    ///
    /// let mut window = Window::new(
    ///     "Test",
    ///     window_width,
    ///     window_height,
    ///     WindowOptions::default(),
    /// )
    /// .unwrap();
    ///
    /// window
    ///     .update_with_buffer(&buffer, buffer_width, buffer_height)
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn update_with_buffer(
        &mut self,
        buffer: &[u32],
        width: usize,
        height: usize,
    ) -> Result<()> {
        self.0.update_rate();
        self.0
            .update_with_buffer_stride(buffer, width, height, width)
    }

    /// Updates the window. Receives any new keyboard or mouse input.
    ///
    /// **Notice:** Only **one** of this function or [`update_with_buffer`](Self::update_with_buffer) should be used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// let mut buffer: Vec<u32> = vec![0; 640 * 400];
    ///
    /// let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    ///
    /// window.update();
    /// ```
    #[inline]
    pub fn update(&mut self) {
        self.0.update_rate();
        self.0.update()
    }

    /// Checks if the window is still open.
    ///
    /// A window can be closed by the user, usually by pressing the close button.
    /// It's up to the program to check whether the window is still open or not.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// while window.is_open() {
    ///     // Update the window here!
    ///     window.update();
    /// }
    /// // The window was closed.
    /// println!("Goodbye!");
    /// ```
    #[inline]
    pub fn is_open(&self) -> bool {
        self.0.is_open()
    }

    /// Sets the position of the window. This is useful if you have
    /// more than one window and want to align them up on the screen
    ///
    /// Sets the position of the window.
    ///
    /// This is useful if you have more than one window and want to line them up on the screen.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Moves the window to the pixel position (20, 20) on the screen.
    /// window.set_position(20, 20);
    /// ```
    #[inline]
    pub fn set_position(&mut self, x: isize, y: isize) {
        self.0.set_position(x, y)
    }

    /// Returns the position of the window.
    ///
    /// This is useful if you want the window's position to persist across sessions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Retrieves the window's current position.
    /// let (x, y) = window.get_position();
    /// ```
    #[inline]
    pub fn get_position(&self) -> (isize, isize) {
        self.0.get_position()
    }

    /// Returns the current size of the window.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Retrieves the window's current size.
    /// let (width, height) = window.get_size();
    /// ```
    #[inline]
    pub fn get_size(&self) -> (usize, usize) {
        self.0.get_size()
    }

    /// Makes the window the topmost window and makes it stay always on top. This is useful if you
    /// want the window to float above all over windows
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Makes the window always be on top.
    /// window.topmost(true);
    /// ```
    #[inline]
    pub fn topmost(&self, topmost: bool) {
        self.0.topmost(topmost)
    }

    /// Sets the background color that is used with update_with_buffer.
    /// In some cases there will be a blank area around the buffer depending on the ScaleMode that has been set.
    /// This color will be used in the in that area.
    /// The function takes 3 parameters in (red, green, blue) and each value is in the range of 0-255 where 255 is the brightest value
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Sets the background color to bright red.
    /// window.set_background_color(255, 0, 0);
    /// ```
    #[inline]
    pub fn set_background_color(&mut self, red: usize, green: usize, blue: usize) {
        let r = clamp(0, red, 255);
        let g = clamp(0, green, 255);
        let b = clamp(0, blue, 255);
        self.0
            .set_background_color(((r << 16) | (g << 8) | b) as u32);
    }

    /// Changes whether or not the cursor image should be shown or if the cursor image
    /// should be invisible inside the window
    /// When creating a new window the default is 'false'
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Makes the cursor invisible.
    /// window.set_cursor_visibility(false);
    /// ```
    #[inline]
    pub fn set_cursor_visibility(&mut self, visibility: bool) {
        self.0.set_cursor_visibility(visibility);
    }

    /// Limits the update rate of polling for new events in order to reduce CPU usage.
    /// The problem of having a tight loop that does something like this
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// loop {
    ///     window.update();
    /// }
    /// ```
    /// Is that lots of CPU time will be spent calling system functions to check for new events in a tight loop making the CPU time go up.
    /// Using `limit_update_rate` minifb will check how much time has passed since the last time and if it's less than the selected time it will sleep for the remainder of it.
    /// This means that if more time has spent than the set time (external code taking longer) minifb will not do any waiting at all so there is no loss in CPU performance with this feature.
    /// By default it's set to 4 milliseconds. Setting this value to None and no waiting will be done
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Make sure that at least 4 milliseconds have passed since the last event poll.
    /// window.limit_update_rate(Some(std::time::Duration::from_millis(4)));
    /// ```
    #[inline]
    pub fn limit_update_rate(&mut self, time: Option<std::time::Duration>) {
        self.0.set_rate(time)
    }

    /// Returns the mouse position relative to the window.
    ///
    /// The coordinate system's origin is at the upper-left corner of the window.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Retrieves the current mouse position, clamped within the window.
    /// window.get_mouse_pos(MouseMode::Clamp).map(|mouse| {
    ///     println!("x {} y {}", mouse.0, mouse.1);
    /// });
    /// ```
    #[inline]
    pub fn get_mouse_pos(&self, mode: MouseMode) -> Option<(f32, f32)> {
        self.0.get_mouse_pos(mode)
    }

    /// Returns the mouse position relative to the window, ignoring any scaling.
    ///
    /// The coordinate system's origin is at the upper-left corner of the window.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Retrieves the current mouse position, clamped within the window, not accounting for any scaling.
    /// window
    ///     .get_unscaled_mouse_pos(MouseMode::Clamp)
    ///     .map(|mouse| {
    ///         println!("x {} y {}", mouse.0, mouse.1);
    ///     });
    /// ```
    #[inline]
    pub fn get_unscaled_mouse_pos(&self, mode: MouseMode) -> Option<(f32, f32)> {
        self.0.get_unscaled_mouse_pos(mode)
    }

    /// Checks if a mouse button is down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// // Checks if the left mouse button is down.
    /// let left_down = window.get_mouse_down(MouseButton::Left);
    /// println!("is left down? {}", left_down)
    /// ```
    #[inline]
    pub fn get_mouse_down(&self, button: MouseButton) -> bool {
        self.0.get_mouse_down(button)
    }

    /// Returns the movement of the scroll wheel.
    ///
    /// Scrolling can mean different things depending on the device being used.
    ///
    /// For example, on a Mac with a trackpad, the "scroll wheel" is two-finger swiping up-down (y-axis) or side-to-side (x-axis).
    ///
    /// When using a mouse, however, scrolling is often only on the y-axis.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.get_scroll_wheel().map(|scroll| {
    ///     println!("scrolling - x {} y {}", scroll.0, scroll.1);
    /// });
    /// ```
    #[inline]
    pub fn get_scroll_wheel(&self) -> Option<(f32, f32)> {
        self.0.get_scroll_wheel()
    }

    /// Changes the cursor style.
    ///
    /// See [`CursorStyle`] for every available style.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.set_cursor_style(CursorStyle::ResizeLeftRight);
    /// ```
    pub fn set_cursor_style(&mut self, cursor: CursorStyle) {
        self.0.set_cursor_style(cursor)
    }

    /// Returns the keys that are currently down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.get_keys().iter().for_each(|key| match key {
    ///     Key::W => println!("holding w"),
    ///     Key::T => println!("holding t"),
    ///     _ => (),
    /// });
    /// ```
    #[inline]
    pub fn get_keys(&self) -> Vec<Key> {
        self.0.get_keys()
    }

    /// Returns the currently pressed keys.
    ///
    /// [`KeyRepeat`] controls whether or not repeated keys should be counted as pressed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window
    ///     .get_keys_pressed(KeyRepeat::No)
    ///     .iter()
    ///     .for_each(|key| match key {
    ///         Key::W => println!("pressed w"),
    ///         Key::T => println!("pressed t"),
    ///         _ => (),
    ///     });
    /// ```
    #[inline]
    pub fn get_keys_pressed(&self, repeat: KeyRepeat) -> Vec<Key> {
        self.0.get_keys_pressed(repeat)
    }

    /// Returns the currently released keys.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.get_keys_released().iter().for_each(|key| match key {
    ///     Key::W => println!("released w"),
    ///     Key::T => println!("released t"),
    ///     _ => (),
    /// });
    /// ```
    #[inline]
    pub fn get_keys_released(&self) -> Vec<Key> {
        self.0.get_keys_released()
    }

    /// Checks if a single key is down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// if window.is_key_down(Key::A) {
    ///     println!("Key A is down");
    /// }
    /// ```
    #[inline]
    pub fn is_key_down(&self, key: Key) -> bool {
        self.0.is_key_down(key)
    }

    /// Checks if a single key was pressed.
    ///
    /// [`KeyRepeat`] controls whether or not repeated keys should be counted as pressed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// if window.is_key_pressed(Key::A, KeyRepeat::No) {
    ///     println!("Key A is down");
    /// }
    /// ```
    #[inline]
    pub fn is_key_pressed(&self, key: Key, repeat: KeyRepeat) -> bool {
        self.0.is_key_pressed(key, repeat)
    }

    /// Checks if a single key was released.
    #[inline]
    pub fn is_key_released(&self, key: Key) -> bool {
        self.0.is_key_released(key)
    }

    /// Sets the delay (in seconds) to wait before keys are repeated.
    ///
    /// The default value is `0.25` seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.set_key_repeat_delay(0.5) // 0.5 seconds before repeat starts
    /// ```
    #[inline]
    pub fn set_key_repeat_delay(&mut self, delay: f32) {
        self.0.set_key_repeat_delay(delay)
    }

    /// Sets the rate (in seconds) at which keys will repeat after the initial delay.
    ///
    /// The default value is `0.05` seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut window = Window::new("Test", 640, 400, WindowOptions::default()).unwrap();
    /// window.set_key_repeat_rate(0.01) // 0.01 seconds between keys
    /// ```
    #[inline]
    pub fn set_key_repeat_rate(&mut self, rate: f32) {
        self.0.set_key_repeat_rate(rate)
    }

    /// Returns `true` if this window is currently the active one.
    #[inline]
    pub fn is_active(&mut self) -> bool {
        self.0.is_active()
    }

    /// Sets the input callback to use when keyboard input is received.
    #[inline]
    pub fn set_input_callback(&mut self, callback: Box<dyn InputCallback>) {
        self.0.set_input_callback(callback)
    }

    /// Adds a menu to the window.
    ///
    /// # Platform-specific behavior
    ///
    /// Menus may behave differently depending on the platform:
    ///
    /// - **Windows**:
    ///   Each window has their own menu and shortcuts are active depending on the active window.
    /// - **macOS**:
    ///   As macOS uses one menu for the entire program, the menu will change depending on which window is active.
    /// - **Linux/BSD/etc**:
    ///   Menus aren't supported as they depend on different window managers and are outside the scope of this library.
    ///   Use [`get_posix_menus`](Self::get_posix_menus) for an alternative.
    #[inline]
    pub fn add_menu(&mut self, menu: &Menu) -> MenuHandle {
        self.0.add_menu(&menu.0)
    }

    /// Removes a menu added with [`add_menu`](Self::add_menu).
    #[inline]
    pub fn remove_menu(&mut self, handle: MenuHandle) {
        self.0.remove_menu(handle)
    }

    /// Returns the window's POSIX menus.
    ///
    /// Will only return menus on POSIX-like systems (Linux, BSD, etc).
    /// On other platforms, `None` will be returned.
    pub fn get_posix_menus(&self) -> Option<&Vec<PosixMenu>> {
        #[cfg(any(target_os = "macos", target_os = "windows", target_arch = "wasm32"))]
        {
            None
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "redox"
        ))]
        {
            self.0.get_posix_menus()
        }
    }

    /// Deprecated. Use [`get_posix_menus`](Self::get_posix_menus) instead.
    #[deprecated(
        since = "0.17.0",
        note = "`get_unix_menus` will be removed in 1.0.0, use `get_posix_menus` instead"
    )]
    #[allow(deprecated)]
    pub fn get_unix_menus(&self) -> Option<&Vec<UnixMenu>> {
        self.get_posix_menus()
    }

    /// Checks if a menu item has been pressed.
    #[inline]
    pub fn is_menu_pressed(&mut self) -> Option<usize> {
        self.0.is_menu_pressed()
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct Modifiers: u32 {
        /// The shift key.
        const SHIFT = 0b0001;
        /// The control key.
        const CTRL = 0b0010;
        /// The alt key.
        const ALT = 0b0100;
        /// The logo key.
        const LOGO = 0b1000;
    }
}

impl Modifiers {
    pub const fn shift(self) -> bool {
        self.intersects(Self::SHIFT)
    }

    pub const fn ctrl(self) -> bool {
        self.intersects(Self::CTRL)
    }

    pub const fn alt(self) -> bool {
        self.intersects(Self::ALT)
    }

    pub const fn logo(self) -> bool {
        self.intersects(Self::LOGO)
    }
}

const MENU_ID_SEPARATOR: usize = 0xffffffff;

/// Deprecated. Use [`PosixMenu`] instead.
#[deprecated(
    since = "0.25.0",
    note = "`UnixMenuItem` will be removed in 1.0.0, use `PosixMenu` instead"
)]
pub type UnixMenu = PosixMenu;

/// Deprecated. Use [`PosixMenuItem`] instead.
#[deprecated(
    since = "0.25.0",
    note = "`UnixMenuItem` will be removed in 1.0.0, use `PosixMenuItem` instead"
)]
pub type UnixMenuItem = PosixMenuItem;

/// Used on POSIX-like systems (Linux, BSD, etc) as menus aren't natively supported there.
///
/// See [`Window::get_posix_menus`].
#[derive(Debug, Clone)]
pub struct PosixMenu {
    /// The name of the menu.
    pub name: String,
    /// A list of items within the menu.
    pub items: Vec<PosixMenuItem>,

    #[doc(hidden)]
    pub handle: MenuHandle,
    #[doc(hidden)]
    pub item_counter: MenuItemHandle,
}

/// Used on POSIX-like systems (Linux, BSD, etc) as menus aren't natively supported there.
///
/// Holds information about an item in a [`PosixMenu`].
#[derive(Debug, Clone)]
pub struct PosixMenuItem {
    /// The item may hold a sub menu.
    pub sub_menu: Option<Box<PosixMenu>>,
    /// The ID of the item.
    ///
    /// Set by the user and reported back when the item is pressed.
    pub id: usize,
    /// The name of the item.
    pub label: String,
    /// Set to `true` if the items is enabled, `false` otherwise.
    pub enabled: bool,
    /// The shortcut key.
    pub key: Key,
    /// The modifiers for the shortcut key.
    pub modifiers: Modifiers,

    #[doc(hidden)]
    pub handle: MenuItemHandle,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[doc(hidden)]
pub struct MenuHandle(pub u64);

#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct MenuItemHandle(pub u64);

/// Holds information about a menu.
pub struct Menu(imp::Menu);

impl fmt::Debug for Menu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Menu").field(&format_args!("..")).finish()
    }
}

impl Menu {
    /// Creates a new menu. Returns an error upon failure.
    pub fn new(name: &str) -> Result<Menu> {
        imp::Menu::new(name).map(Menu)
    }

    /// Destroys the menu. Currently unimplemented.
    #[inline]
    pub fn destroy_menu(&mut self) {
        //self.0.destroy_menu()
    }

    /// Adds a sub menu to the menu.
    #[inline]
    pub fn add_sub_menu(&mut self, name: &str, menu: &Menu) {
        self.0.add_sub_menu(name, &menu.0)
    }

    /// Adds a separator to the menu.
    pub fn add_separator(&mut self) {
        self.add_menu_item(&MenuItem {
            id: MENU_ID_SEPARATOR,
            ..MenuItem::default()
        });
    }

    /// Adds an item to the menu.
    #[inline]
    pub fn add_menu_item(&mut self, item: &MenuItem) -> MenuItemHandle {
        self.0.add_menu_item(item)
    }

    /// Begins building an item to be added to the menu.
    ///
    /// [`MenuItem::build`] must be called to add the finished item.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut menu = Menu::new("test").unwrap();
    /// menu.add_item("test", 1).shortcut(Key::A, 0).build()
    /// # ;
    /// ```
    #[inline]
    pub fn add_item(&mut self, name: &str, id: usize) -> MenuItem {
        MenuItem {
            id,
            label: name.to_owned(),
            menu: Some(self),
            ..MenuItem::default()
        }
    }

    /// Removes an item from the menu.
    #[inline]
    pub fn remove_item(&mut self, item: &MenuItemHandle) {
        self.0.remove_item(item)
    }
}

/// Holds information about an item in a [`Menu`].
#[derive(Debug)]
pub struct MenuItem<'a> {
    pub id: usize,
    pub label: String,
    pub enabled: bool,
    pub key: Key,
    pub modifiers: Modifiers,

    #[doc(hidden)]
    pub menu: Option<&'a mut Menu>,
}

impl<'a> Default for MenuItem<'a> {
    fn default() -> Self {
        MenuItem {
            id: MENU_ID_SEPARATOR,
            label: String::new(),
            enabled: true,
            key: Key::Unknown,
            modifiers: Modifiers::empty(),
            menu: None,
        }
    }
}

impl<'a> Clone for MenuItem<'a> {
    fn clone(&self) -> Self {
        MenuItem {
            id: self.id,
            label: self.label.clone(),
            enabled: self.enabled,
            key: self.key,
            modifiers: self.modifiers,
            menu: None,
        }
    }
}

impl<'a> MenuItem<'a> {
    /// Creates a new menu item.
    pub fn new(name: &str, id: usize) -> MenuItem {
        MenuItem {
            id,
            label: name.to_owned(),
            ..MenuItem::default()
        }
    }

    /// Sets a shortcut key and modifier.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut menu = Menu::new("test").unwrap();
    /// menu.add_item("test", 1).shortcut(Key::A, 0).build()
    /// # ;
    /// ```
    #[inline]
    pub fn shortcut(self, key: Key, modifiers: Modifiers) -> Self {
        MenuItem {
            key,
            modifiers,
            ..self
        }
    }

    /// Makes the menu item a separator.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut menu = Menu::new("test").unwrap();
    /// menu.add_item("", 0).separator().build()
    /// # ;
    /// ```
    ///
    /// Note that it is often more convenient to instead call [`Menu::add_separator`].
    #[inline]
    pub fn separator(self) -> Self {
        MenuItem {
            id: MENU_ID_SEPARATOR,
            ..self
        }
    }

    /// Makes the menu item appear enabled or disabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut menu = Menu::new("test").unwrap();
    /// menu.add_item("test", 1).enabled(false).build()
    /// # ;
    /// ```
    #[inline]
    pub fn enabled(self, enabled: bool) -> Self {
        MenuItem { enabled, ..self }
    }

    /// Must be called to add a menu item started with [`Menu::add_item`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use minifb::*;
    /// # let mut menu = Menu::new("test").unwrap();
    /// menu.add_item("test", 1).enabled(false).build()
    /// # ;
    /// ```
    #[inline]
    pub fn build(&mut self) -> MenuItemHandle {
        let t = self.clone();
        if let Some(ref mut menu) = self.menu {
            menu.0.add_menu_item(&t)
        } else {
            MenuItemHandle(0)
        }
    }
}

pub fn clamp<T: PartialOrd>(low: T, value: T, high: T) -> T {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}
