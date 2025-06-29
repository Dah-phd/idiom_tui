#[cfg(feature = "crossterm_backend")]
mod crossterm_backend;
mod style;
use super::layout::Rect;
#[cfg(feature = "crossterm_backend")]
pub use crossterm_backend::{background_rgb, parse_raw_rgb, pull_color, serialize_rgb, CrossTerm};
use std::{
    fmt::{Debug, Display},
    io::{Result, Write},
};
pub use style::StyleExt;

pub const ERR_MSG: &str = "Rendering (Stdout) Err:";

/// If stdout is returning errors the program should crash -> use expect
// impl all utilities although not all are used
pub trait Backend: Write + Sized + Debug + PartialEq + Default {
    type Style: Sized + PartialEq + Debug + Clone;
    type Color: Sized + PartialEq + Debug + Clone;

    fn init() -> Self;
    fn exit() -> std::io::Result<()>;
    /// get whole screen as rect
    fn screen() -> Result<Rect>;
    /// stop updates allowing to build buffer
    fn freeze(&mut self);
    /// restore updates allowing to render buffer
    fn unfreeze(&mut self);
    fn flush_buf(&mut self);
    /// clears from cursor until the End Of Line
    fn clear_to_eol(&mut self);
    /// clears current cursor line
    fn clear_line(&mut self);
    fn clear_all(&mut self);
    /// stores the cursor
    fn save_cursor(&mut self);
    /// restores cursor position
    fn restore_cursor(&mut self);
    /// sets the style for the print/print at
    fn set_style(&mut self, style: Self::Style);
    fn get_style(&mut self) -> Self::Style;
    fn to_set_style(&mut self);
    /// update existing style if exists otherwise sets it to the new one
    /// mods will be taken from updating and will replace fg and bg if present
    fn update_style(&mut self, style: Self::Style);
    /// adds foreground to the already set style
    fn set_fg(&mut self, color: Option<Self::Color>);
    /// adds background to the already set style
    fn set_bg(&mut self, color: Option<Self::Color>);
    /// restores the style of the writer to default
    fn reset_style(&mut self);
    /// sends the cursor to location
    fn go_to(&mut self, row: u16, col: u16);
    /// direct adding cursor at location - no buffer queing
    fn render_cursor_at(&mut self, row: u16, col: u16);
    /// direct showing cursor - no buffer queing
    fn show_cursor(&mut self);
    /// direct hiding cursor - no buffer queing
    fn hide_cursor(&mut self);
    /// print text at current location - default styling
    fn print<D: Display>(&mut self, text: D);
    /// goes to location and prints text
    fn print_at<D: Display>(&mut self, row: u16, col: u16, text: D);
    /// prints styled text without affecting the writer set style
    fn print_styled<D: Display>(&mut self, text: D, style: Self::Style);
    /// goes to location and prints styled text without affecting the writer set style
    fn print_styled_at<D: Display>(&mut self, row: u16, col: u16, text: D, style: Self::Style);
    /// padding with empty space
    fn pad(&mut self, width: usize);
    /// padding with empty space styled
    fn pad_styled(&mut self, width: usize, style: Self::Style);
    /// merge styles
    fn merge_style(left: Self::Style, right: Self::Style) -> Self::Style;
    /// Self::Style with revers attr
    fn reversed_style() -> Self::Style;
    /// Self::Style with bold attr
    fn bold_style() -> Self::Style;
    /// Self::Style with ital attr
    fn ital_style() -> Self::Style;
    /// Self::Style with slow blink attr
    fn slow_blink_style() -> Self::Style;
    /// Self::Style with bold attr
    fn underline_style(color: Option<Self::Color>) -> Self::Style;
    /// Self::Style with bold attr
    fn undercurle_style(color: Option<Self::Color>) -> Self::Style;
    /// Self::Style from forground color
    fn fg_style(color: Self::Color) -> Self::Style;
    /// Self::Style from background color
    fn bg_style(color: Self::Color) -> Self::Style;
}

#[cfg(test)]
mod test;

#[cfg(test)]
pub use test::{MockedBackend, MockedStyle};
