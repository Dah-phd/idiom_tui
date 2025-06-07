use std::io::Write;

use super::{style::StyleExt, Backend};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct MockedStyle {
    fg: Option<usize>,
    bg: Option<usize>,
    attrs: Vec<isize>,
}

impl StyleExt for MockedStyle {
    type Attribute = isize;
    type Color = usize;

    fn add_bold(&mut self) {
        self.attrs.push(1);
    }

    fn add_ital(&mut self) {
        self.attrs.push(2);
    }

    fn add_reverse(&mut self) {
        self.attrs.push(3);
    }

    fn add_slowblink(&mut self) {
        self.attrs.push(4);
    }

    fn bg(color: Self::Color) -> Self {
        Self {
            bg: Some(color),
            ..Default::default()
        }
    }

    fn bold() -> Self {
        Self {
            attrs: vec![1],
            ..Default::default()
        }
    }

    fn drop_bg(&mut self) {
        self.bg = None;
    }

    fn fg(color: Self::Color) -> Self {
        Self {
            fg: Some(color),
            ..Default::default()
        }
    }

    fn ital() -> Self {
        Self {
            attrs: vec![2],
            ..Default::default()
        }
    }

    fn reset_mods(&mut self) {
        self.attrs.clear();
    }

    fn reversed() -> Self {
        Self {
            attrs: vec![3],
            ..Default::default()
        }
    }

    fn set_attr(&mut self, attr: Self::Attribute) {
        self.attrs.push(attr);
    }

    fn set_bg(&mut self, color: Option<Self::Color>) {
        self.bg = color;
    }

    fn set_fg(&mut self, color: Option<Self::Color>) {
        self.fg = color;
    }

    fn slowblink() -> Self {
        Self {
            attrs: vec![4],
            ..Default::default()
        }
    }

    fn undercurle(&mut self, _: Option<Self::Color>) {
        self.attrs.push(5);
    }

    fn undercurled(_: Option<Self::Color>) -> Self {
        Self {
            attrs: vec![5],
            ..Default::default()
        }
    }

    fn underline(&mut self, _: Option<Self::Color>) {
        self.attrs.push(6);
    }

    fn underlined(_: Option<Self::Color>) -> Self {
        Self {
            attrs: vec![6],
            ..Default::default()
        }
    }

    fn unset_attr(&mut self, attr: Self::Attribute) {
        self.attrs.retain(|x| x != &attr);
    }

    fn update(&mut self, rhs: Self) {
        self.bg = rhs.bg;
        self.fg = rhs.fg;
        self.attrs.extend(rhs.attrs);
    }

    fn with_bg(self, color: Self::Color) -> Self {
        Self {
            bg: Some(color),
            ..Default::default()
        }
    }

    fn with_fg(self, color: Self::Color) -> Self {
        Self {
            fg: Some(color),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct MockedBackend {
    pub data: Vec<(MockedStyle, String)>,
    pub default_style: MockedStyle,
}

impl MockedBackend {
    pub fn detached_hide_cursor() {}

    pub fn detached_show_cursor() {}
}

impl PartialEq for MockedBackend {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Backend for MockedBackend {
    type Style = MockedStyle;
    type Color = usize;

    fn init() -> Self {
        Self {
            data: Vec::new(),
            default_style: MockedStyle::default(),
        }
    }

    fn exit() -> std::io::Result<()> {
        Ok(())
    }

    fn freeze(&mut self) {
        self.data
            .push((MockedStyle::default(), String::from("<<freeze>>")));
    }

    fn unfreeze(&mut self) {
        self.data
            .push((MockedStyle::default(), String::from("<<unfreeze>>")));
    }

    /// force flush buffer if writing small amount of data
    fn flush_buf(&mut self) {}

    fn clear_all(&mut self) {
        self.data
            .push((MockedStyle::default(), String::from("<<clear all>>")));
    }
    fn clear_line(&mut self) {
        self.data
            .push((MockedStyle::default(), String::from("<<clear line>>")));
    }
    fn clear_to_eol(&mut self) {
        self.data
            .push((MockedStyle::default(), String::from("<<clear EOL>>")));
    }

    fn get_style(&mut self) -> Self::Style {
        self.default_style.clone()
    }

    fn go_to(&mut self, row: u16, col: u16) {
        self.data.push((
            MockedStyle::default(),
            format!("<<go to row: {row} col: {col}>>"),
        ))
    }

    fn hide_cursor(&mut self) {}

    fn print<D: std::fmt::Display>(&mut self, text: D) {
        self.data
            .push((self.default_style.clone(), text.to_string()));
    }

    fn print_at<D: std::fmt::Display>(&mut self, row: u16, col: u16, text: D) {
        self.go_to(row, col);
        self.print(text)
    }
    fn print_styled<D: std::fmt::Display>(&mut self, text: D, style: Self::Style) {
        self.data.push((style, text.to_string()));
    }

    fn print_styled_at<D: std::fmt::Display>(
        &mut self,
        row: u16,
        col: u16,
        text: D,
        style: MockedStyle,
    ) {
        self.go_to(row, col);
        self.print_styled(text, style);
    }

    fn render_cursor_at(&mut self, row: u16, col: u16) {
        self.data.push((
            self.default_style.clone(),
            format!("<<draw cursor row: {row} col: {col}>>"),
        ));
    }

    fn reset_style(&mut self) {
        self.default_style = MockedStyle::default();
        self.data
            .push((self.default_style.clone(), String::from("<<reset style>>")));
    }

    fn restore_cursor(&mut self) {
        self.data.push((
            self.default_style.clone(),
            String::from("<<restored cursor>>"),
        ))
    }

    fn save_cursor(&mut self) {
        self.data
            .push((self.default_style.clone(), String::from("<<saved cursor>>")));
    }

    fn screen() -> std::io::Result<crate::layout::Rect> {
        Ok(crate::layout::Rect::new(0, 0, 120, 60))
    }

    fn set_bg(&mut self, color: Option<Self::Color>) {
        self.default_style.set_bg(color);
        self.data.push((
            self.default_style.clone(),
            format!("<<set bg {:?}>>", color),
        ));
    }

    fn set_fg(&mut self, color: Option<Self::Color>) {
        self.default_style.set_fg(color);
        self.data.push((
            self.default_style.clone(),
            format!("<<set fg {:?}>>", color),
        ));
    }

    fn set_style(&mut self, style: MockedStyle) {
        self.default_style = style;
        self.data
            .push((self.default_style.clone(), "<<set style>>".to_string()))
    }

    fn show_cursor(&mut self) {}
    // self.data.push((self.default_style, String::from("<<show cursor>>")));

    fn to_set_style(&mut self) {
        self.data
            .push((self.default_style.clone(), String::from("<<set style>>")));
    }

    fn update_style(&mut self, style: MockedStyle) {
        self.default_style.update(style);
        self.data.push((
            self.default_style.clone(),
            String::from("<<updated style>>"),
        ))
    }

    fn pad(&mut self, width: usize) {
        self.data.push((
            self.default_style.clone(),
            format!("<<padding: {:?}>>", width),
        ))
    }

    fn pad_styled(&mut self, width: usize, style: MockedStyle) {
        self.data.push((
            self.default_style.clone(),
            format!("<<padding: {:?}, styled: {:?}>>", width, style),
        ))
    }

    fn merge_style(mut left: Self::Style, right: Self::Style) -> Self::Style {
        left.update(right);
        left
    }

    fn reversed_style() -> Self::Style {
        Self::Style::reversed()
    }

    fn bold_style() -> Self::Style {
        Self::Style::bold()
    }

    fn slow_blink_style() -> Self::Style {
        Self::Style::slowblink()
    }

    fn ital_style() -> Self::Style {
        Self::Style::ital()
    }

    fn undercurle_style(color: Option<Self::Color>) -> Self::Style {
        Self::Style::undercurled(color)
    }

    fn underline_style(color: Option<Self::Color>) -> Self::Style {
        Self::Style::underlined(color)
    }

    fn bg_style(color: Self::Color) -> Self::Style {
        Self::Style::bg(color)
    }

    fn fg_style(color: Self::Color) -> Self::Style {
        Self::Style::fg(color)
    }
}

impl Write for MockedBackend {
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, mut _buf: &[u8]) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }
}

impl MockedBackend {
    pub fn unwrap(self) -> Vec<(MockedStyle, String)> {
        self.data
    }

    pub fn drain(&mut self) -> Vec<(MockedStyle, String)> {
        std::mem::take(&mut self.data)
    }
}
