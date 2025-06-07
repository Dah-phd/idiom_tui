use super::{style::StyleExt, ERR_MSG};
use crossterm::style::Color;
use crossterm::style::{Attribute, Attributes};
use crossterm::{
    cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
    execute, queue,
    style::{ContentStyle, Print, ResetColor, SetStyle},
    terminal::{size, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate},
};
use serde_json::{Map, Value};
use std::{collections::HashMap, fmt::Debug};
use std::{
    fmt::Display,
    io::{Stdout, Write},
};

use super::super::layout::Rect;

use super::Backend;

/// Thin wrapper around rendering framework, allowing easy switching of backend
/// If stdout gets an error Backend will crash the program as rendering is to priority
/// Add cfg and new implementation of the wrapper to make the backend swichable
/// Main reason is to clear out the issue with PrintStyled on CrossTerm
#[derive(Debug)]
pub struct CrossTerm {
    writer: Stdout, // could be moved to locked state for performance but current frame generation is about 200 µs
    default_styled: Option<ContentStyle>,
}

impl Default for CrossTerm {
    fn default() -> Self {
        Self::init()
    }
}

impl PartialEq for CrossTerm {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Write for CrossTerm {
    #[inline(always)]
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(buf)
    }

    #[inline(always)]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.writer.write_fmt(fmt)
    }
}

impl CrossTerm {
    pub fn detached_hide_cursor() {
        queue!(std::io::stdout(), Show).expect(ERR_MSG);
    }

    pub fn detached_show_cursor() {
        queue!(std::io::stdout(), Hide).expect(ERR_MSG);
    }
}

impl Backend for CrossTerm {
    type Style = ContentStyle;
    type Color = Color;

    #[inline]
    fn init() -> Self {
        init_terminal().expect(ERR_MSG);
        Self {
            writer: std::io::stdout(),
            default_styled: None,
        }
    }

    #[inline]
    fn exit() -> std::io::Result<()> {
        graceful_exit()
    }

    /// get whole screen as rect
    #[inline]
    fn screen() -> std::io::Result<Rect> {
        size().map(Rect::from)
    }

    /// freeze screen allowing to build buffer
    #[inline]
    fn freeze(&mut self) {
        execute!(self, BeginSynchronizedUpdate).expect(ERR_MSG);
    }

    /// unfreeze allowing the buffer to render
    #[inline]
    fn unfreeze(&mut self) {
        execute!(self, EndSynchronizedUpdate).expect(ERR_MSG);
    }

    /// flushs buffer with panic on error
    #[inline]
    fn flush_buf(&mut self) {
        self.writer.flush().expect(ERR_MSG);
    }

    /// clears from cursor until the End Of Line
    #[inline]
    fn clear_to_eol(&mut self) {
        queue!(self, Clear(ClearType::UntilNewLine)).expect(ERR_MSG);
    }

    /// clears current cursor line
    #[inline]
    fn clear_line(&mut self) {
        queue!(self, Clear(ClearType::CurrentLine)).expect(ERR_MSG);
    }

    #[inline]
    fn clear_all(&mut self) {
        queue!(self, Clear(ClearType::All)).expect(ERR_MSG);
    }

    /// stores the cursor
    #[inline]
    fn save_cursor(&mut self) {
        execute!(self, SavePosition).expect(ERR_MSG);
    }

    /// restores cursor position
    #[inline]
    fn restore_cursor(&mut self) {
        queue!(self, RestorePosition).expect(ERR_MSG);
    }

    /// sets the style for the print/print at
    #[inline]
    fn set_style(&mut self, style: ContentStyle) {
        self.default_styled.replace(style);
        queue!(self, ResetColor, SetStyle(style)).expect(ERR_MSG);
    }

    #[inline]
    fn get_style(&mut self) -> ContentStyle {
        self.default_styled.unwrap_or_default()
    }

    #[inline]
    fn to_set_style(&mut self) {
        match self.default_styled {
            Some(style) => queue!(self, ResetColor, SetStyle(style)),
            None => queue!(self, ResetColor),
        }
        .expect(ERR_MSG);
    }

    /// update existing style if exists otherwise sets it to the new one
    /// mods will be taken from updating and will replace fg and bg if present
    #[inline]
    fn update_style(&mut self, style: ContentStyle) {
        if let Some(current) = self.default_styled.as_mut() {
            current.update(style);
        } else {
            self.default_styled.replace(style);
        };
        self.to_set_style();
    }

    /// adds foreground to the already set style
    #[inline]
    fn set_fg(&mut self, color: Option<Color>) {
        if let Some(current) = self.default_styled.as_mut() {
            current.set_fg(color);
        } else if let Some(color) = color {
            self.default_styled.replace(ContentStyle::fg(color));
        };
        self.to_set_style()
    }

    /// adds background to the already set style
    #[inline]
    fn set_bg(&mut self, color: Option<Color>) {
        if let Some(current) = self.default_styled.as_mut() {
            current.set_bg(color);
        } else if let Some(color) = color {
            let style = ContentStyle::bg(color);
            self.default_styled.replace(style);
        }
        self.to_set_style();
    }

    /// restores the style of the writer to default
    #[inline]
    fn reset_style(&mut self) {
        self.default_styled = None;
        queue!(self, ResetColor).expect(ERR_MSG);
    }

    /// sends the cursor to location
    #[inline]
    fn go_to(&mut self, row: u16, col: u16) {
        queue!(self, MoveTo(col, row)).expect(ERR_MSG);
    }

    /// direct adding cursor at location - no buffer queing
    #[inline]
    fn render_cursor_at(&mut self, row: u16, col: u16) {
        queue!(self, MoveTo(col, row), Show).expect(ERR_MSG);
    }

    /// direct showing cursor - no buffer queing
    #[inline]
    fn show_cursor(&mut self) {
        queue!(self, Show).expect(ERR_MSG);
    }

    /// direct hiding cursor - no buffer queing
    #[inline]
    fn hide_cursor(&mut self) {
        queue!(self, Hide).expect(ERR_MSG);
    }

    #[inline]
    fn print<D: Display>(&mut self, text: D) {
        queue!(self, Print(text)).expect(ERR_MSG);
    }

    /// goes to location and prints text
    #[inline]
    fn print_at<D: Display>(&mut self, row: u16, col: u16, text: D) {
        queue!(self, MoveTo(col, row), Print(text)).expect(ERR_MSG);
    }

    /// prints styled text without affecting the writer set style
    #[inline]
    fn print_styled<D: Display>(&mut self, text: D, style: ContentStyle) {
        match self.default_styled {
            Some(restore_style) => queue!(
                self,
                SetStyle(style),
                Print(text),
                ResetColor,
                SetStyle(restore_style),
            ),
            None => queue!(self, SetStyle(style), Print(text), ResetColor,),
        }
        .expect(ERR_MSG);
    }

    /// goes to location and prints styled text without affecting the writer set style
    #[inline]
    fn print_styled_at<D: Display>(&mut self, row: u16, col: u16, text: D, style: ContentStyle) {
        if let Some(restore_style) = self.default_styled {
            queue!(
                self,
                SetStyle(style),
                MoveTo(col, row),
                Print(text),
                ResetColor,
                SetStyle(restore_style),
            )
        } else {
            queue!(
                self,
                SetStyle(style),
                MoveTo(col, row),
                Print(text),
                ResetColor,
            )
        }
        .expect(ERR_MSG);
    }

    #[inline]
    fn pad(&mut self, width: usize) {
        queue!(self, Print(format!("{:width$}", ""))).expect(ERR_MSG);
    }

    #[inline]
    fn pad_styled(&mut self, width: usize, style: ContentStyle) {
        let text = format!("{:width$}", "");
        match self.default_styled {
            Some(restore_style) => queue!(
                self,
                SetStyle(style),
                Print(text),
                ResetColor,
                SetStyle(restore_style)
            ),
            None => queue!(self, SetStyle(style), Print(text), ResetColor),
        }
        .expect(ERR_MSG);
    }

    #[inline]
    fn merge_style(mut left: ContentStyle, right: ContentStyle) -> ContentStyle {
        left.update(right);
        left
    }

    #[inline]
    fn reversed_style() -> Self::Style {
        Self::Style::reversed()
    }

    #[inline]
    fn bold_style() -> Self::Style {
        Self::Style::bold()
    }

    #[inline]
    fn slow_blink_style() -> Self::Style {
        Self::Style::slowblink()
    }

    #[inline]
    fn ital_style() -> Self::Style {
        Self::Style::ital()
    }

    #[inline]
    fn undercurle_style(color: Option<Self::Color>) -> Self::Style {
        Self::Style::undercurled(color)
    }

    #[inline]
    fn underline_style(color: Option<Self::Color>) -> Self::Style {
        Self::Style::underlined(color)
    }

    fn fg_style(color: Self::Color) -> Self::Style {
        Self::Style::fg(color)
    }

    fn bg_style(color: Self::Color) -> Self::Style {
        Self::Style::bg(color)
    }
}

impl Drop for CrossTerm {
    fn drop(&mut self) {
        let _ = CrossTerm::exit();
    }
}

fn init_terminal() -> std::io::Result<()> {
    // Ensures panics are retported
    std::panic::set_hook(Box::new(|info| {
        let _ = graceful_exit();
        eprintln!("{info}");
    }));
    // Init terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::terminal::DisableLineWrap,
        crossterm::style::ResetColor,
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableBracketedPaste,
        #[cfg(not(windows))]
        crossterm::event::PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
        ),
        crossterm::cursor::Hide,
    )?;
    Ok(())
}

fn graceful_exit() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        #[cfg(not(windows))]
        crossterm::event::PopKeyboardEnhancementFlags,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::terminal::EnableLineWrap,
        crossterm::style::ResetColor,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste,
        crossterm::cursor::Show,
    )?;
    Ok(())
}

impl StyleExt for ContentStyle {
    type Attribute = Attribute;
    type Color = Color;
    #[inline]
    fn update(&mut self, rhs: Self) {
        if let Some(c) = rhs.foreground_color {
            self.foreground_color.replace(c);
        }
        if let Some(c) = rhs.background_color {
            self.background_color.replace(c);
        }
        if let Some(c) = rhs.underline_color {
            self.underline_color.replace(c);
        }
        self.attributes = rhs.attributes;
    }

    fn set_attr(&mut self, attr: Attribute) {
        self.attributes.set(attr);
    }

    fn unset_attr(&mut self, attr: Attribute) {
        self.attributes.unset(attr);
    }

    #[inline]
    fn with_fg(mut self, color: Color) -> Self {
        self.foreground_color = Some(color);
        self
    }

    #[inline]
    fn set_fg(&mut self, color: Option<Color>) {
        self.foreground_color = color;
    }

    #[inline]
    fn fg(color: Color) -> Self {
        ContentStyle {
            foreground_color: Some(color),
            background_color: None,
            underline_color: None,
            attributes: Attributes::default(),
        }
    }

    #[inline]
    fn with_bg(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    #[inline]
    fn set_bg(&mut self, color: Option<Color>) {
        self.background_color = color;
    }

    #[inline]
    fn bg(color: Color) -> Self {
        ContentStyle {
            background_color: Some(color),
            foreground_color: None,
            underline_color: None,
            attributes: Attributes::default(),
        }
    }

    #[inline]
    fn drop_bg(&mut self) {
        self.background_color = None;
    }

    #[inline]
    fn add_slowblink(&mut self) {
        self.attributes.set(Attribute::SlowBlink);
    }

    #[inline]
    fn slowblink() -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: None,
            attributes: Attribute::SlowBlink.into(),
        }
    }

    #[inline]
    fn add_bold(&mut self) {
        self.attributes.set(Attribute::Bold);
    }

    #[inline]
    fn bold() -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: None,
            attributes: Attribute::Bold.into(),
        }
    }

    #[inline]
    fn add_ital(&mut self) {
        self.attributes.set(Attribute::Italic);
    }

    #[inline]
    fn ital() -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: None,
            attributes: Attribute::Italic.into(),
        }
    }

    #[inline]
    fn add_reverse(&mut self) {
        self.attributes.set(Attribute::Reverse);
    }

    #[inline]
    fn reversed() -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: None,
            attributes: Attribute::Reverse.into(),
        }
    }

    #[inline]
    fn reset_mods(&mut self) {
        self.attributes = Attributes::default();
        self.underline_color = None;
    }

    #[inline]
    fn undercurle(&mut self, color: Option<Color>) {
        self.attributes.set(Attribute::Undercurled);
        self.underline_color = color;
    }

    #[inline]
    fn undercurled(color: Option<Color>) -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: color,
            attributes: Attribute::Undercurled.into(),
        }
    }

    #[inline]
    fn underline(&mut self, color: Option<Color>) {
        self.attributes.set(Attribute::Underlined);
        self.underline_color = color;
    }

    #[inline]
    fn underlined(color: Option<Color>) -> Self {
        ContentStyle {
            background_color: None,
            foreground_color: None,
            underline_color: color,
            attributes: Attribute::Underlined.into(),
        }
    }
}

#[cfg(not(test))]
#[inline]
pub fn background_rgb() -> Option<(u8, u8, u8)> {
    #[cfg(unix)]
    if let Some(result) = query_bg_color() {
        return Some(result);
    }
    env_rgb_color()
}

#[cfg(test)]
pub fn background_rgb() -> Option<(u8, u8, u8)> {
    None
}

#[allow(dead_code)] // test setup causes the function to be detected as unused
#[cfg(unix)]
fn query_bg_color() -> Option<(u8, u8, u8)> {
    let s = xterm_query::query_osc("\x1b]11;?\x07", 100_u16).ok()?;
    match s.strip_prefix("]11;rgb:") {
        Some(raw_color) if raw_color.len() >= 14 => Some((
            u8::from_str_radix(&raw_color[0..2], 16).ok()?,
            u8::from_str_radix(&raw_color[5..7], 16).ok()?,
            u8::from_str_radix(&raw_color[10..12], 16).ok()?,
        )),
        _ => None,
    }
}

#[allow(dead_code)] // test setup causes the function to be detected as unused
fn env_rgb_color() -> Option<(u8, u8, u8)> {
    let color_config = std::env::var("COLORFGBG").ok()?;
    let token: Vec<&str> = color_config.split(';').collect();
    let bg = match token.len() {
        2 => token[1],
        3 => token[2],
        _ => {
            return None;
        }
    };
    let code = bg.parse().ok()?;
    let coolor::Rgb { r, g, b } = coolor::AnsiColor { code }.to_rgb();
    Some((r, g, b))
}

pub fn serialize_rgb(r: u8, g: u8, b: u8) -> HashMap<&'static str, [u8; 3]> {
    let mut rgb = HashMap::new();
    rgb.insert("rgb", [r, g, b]);
    rgb
}

#[inline]
pub fn pull_color(map: &mut Map<String, Value>, key: &str) -> Option<Result<Color, String>> {
    map.remove(key).map(parse_color)
}

pub fn parse_color(obj: Value) -> Result<Color, String> {
    match obj {
        Value::String(data) => from_str(&data).map_err(|e| e.to_string()),
        Value::Object(map) => {
            if let Some(Value::Array(rgb_value)) =
                map.get("rgb").or(map.get("Rgb").or(map.get("RGB")))
            {
                if rgb_value.len() == 3 {
                    let b = object_to_u8(rgb_value[2].clone())
                        .ok_or("Failed to parse B in RGB color")?;
                    let g = object_to_u8(rgb_value[1].clone())
                        .ok_or("Failed to parse G in RGB color")?;
                    let r = object_to_u8(rgb_value[0].clone())
                        .ok_or("Failed to parse R in RGB color")?;
                    return Ok(Color::Rgb { r, g, b });
                }
            };
            Err(String::from("When representing Color as Object(Map) - should be {\"rgb\": [number, number, number]}!"))
        }
        _ => Err(String::from("Color definition should be String or Object!")),
    }
}

pub fn parse_raw_rgb(map: Value) -> Result<(u8, u8, u8), String> {
    if let Some(Value::Array(rgb_value)) = map.get("rgb").or(map.get("Rgb").or(map.get("RGB"))) {
        if rgb_value.len() == 3 {
            let b = object_to_u8(rgb_value[2].clone()).ok_or("Failed to parse B in RGB color")?;
            let g = object_to_u8(rgb_value[1].clone()).ok_or("Failed to parse G in RGB color")?;
            let r = object_to_u8(rgb_value[0].clone()).ok_or("Failed to parse R in RGB color")?;
            return Ok((r, g, b));
        }
    };
    Err(String::from(
        "When representing Color as Object(Map) - should be {\"rgb\": [number, number, number]}!",
    ))
}

pub fn object_to_u8(obj: Value) -> Option<u8> {
    match obj {
        Value::Number(num) => Some(num.as_u64()? as u8),
        Value::String(string) => string.parse().ok(),
        _ => None,
    }
}

fn from_str(s: &str) -> Result<Color, ParseColorError> {
    Ok(
        // There is a mix of different color names and formats in the wild.
        // This is an attempt to support as many as possible.
        match s
            .to_lowercase()
            .replace([' ', '-', '_'], "")
            .replace("bright", "light")
            .replace("grey", "gray")
            .replace("silver", "gray")
            .replace("lightblack", "darkgray")
            .replace("lightwhite", "white")
            .replace("lightgray", "white")
            .as_ref()
        {
            "reset" => Color::Reset,
            "black" => Color::Black,
            "red" => Color::DarkRed,
            "lightred" => Color::Red,
            "green" => Color::DarkGreen,
            "lightgreen" => Color::Green,
            "yellow" => Color::DarkYellow,
            "lightyellow" => Color::Yellow,
            "blue" => Color::DarkBlue,
            "lightblue" => Color::Blue,
            "magenta" => Color::DarkMagenta,
            "lightmagenta" => Color::Magenta,
            "cyan" => Color::DarkCyan,
            "lightcyan" => Color::Cyan,
            "gray" => Color::Grey,
            "darkgray" => Color::DarkGrey,
            "white" => Color::White,
            _ => {
                if let Ok(index) = s.parse::<u8>() {
                    Color::AnsiValue(index)
                } else if let (Ok(r), Ok(g), Ok(b)) = {
                    if !s.starts_with('#') || s.len() != 7 {
                        return Err(ParseColorError);
                    }
                    (
                        u8::from_str_radix(&s[1..3], 16),
                        u8::from_str_radix(&s[3..5], 16),
                        u8::from_str_radix(&s[5..7], 16),
                    )
                } {
                    Color::Rgb { r, g, b }
                } else {
                    return Err(ParseColorError);
                }
            }
        },
    )
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ParseColorError;

impl std::fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse Colors")
    }
}

impl std::error::Error for ParseColorError {}
