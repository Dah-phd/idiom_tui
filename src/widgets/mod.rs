mod state;

use crate::{
    backend::Backend,
    layout::{IterLines, Line, RectIter},
    StrChunks, UTF8Safe, WriteChunks,
};
pub use state::State;
use std::fmt::Display;
use unicode_width::UnicodeWidthChar;

/// Trait that allows faster rendering without checks and can reduce complexity
pub trait Writable<B: Backend>: Display {
    /// check if the line can be rendered as ascii - no control chars should be included
    fn is_simple(&self) -> bool;
    /// width when rendered
    fn width(&self) -> usize;
    fn char_len(&self) -> usize;
    fn len(&self) -> usize;
    /// directly render no checks or bounds
    fn print(&self, backend: &mut B);
    /// prints bounded by line
    fn print_at(&self, line: Line, backend: &mut B);
    /// wraps within rect
    fn wrap(&self, lines: &mut impl IterLines, backend: &mut B);
    /// # Safety
    /// print truncated
    unsafe fn print_truncated(&self, width: usize, backend: &mut B);
    /// # Safety
    /// print truncated start
    unsafe fn print_truncated_start(&self, width: usize, backend: &mut B);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Represents word with additional meta data such as width, style and number of chars, useful when rendering multiple times the same string
#[derive(Clone, PartialEq)]
pub struct Text<B: Backend> {
    text: String,
    char_len: usize,
    width: usize,
    style: Option<<B as Backend>::Style>,
}

impl<B: Backend> Text<B> {
    #[inline]
    pub fn new(text: String, style: Option<<B as Backend>::Style>) -> Self {
        Self {
            char_len: text.char_len(),
            width: text.width(),
            style,
            text,
        }
    }

    #[inline]
    pub fn raw(text: String) -> Self {
        Self {
            char_len: text.char_len(),
            width: text.width(),
            style: None,
            text,
        }
    }

    #[inline]
    pub fn style(&self) -> Option<<B as Backend>::Style> {
        self.style.clone()
    }

    #[inline]
    pub fn set_style(&mut self, style: Option<<B as Backend>::Style>) {
        self.style = style;
    }

    #[inline]
    pub fn simple_wrap(&self, lines: &mut RectIter, backend: &mut B) {
        let max_width = match lines.move_cursor(backend) {
            Some(width) => width,
            None => return,
        };
        if max_width > self.width {
            match self.style.clone() {
                Some(style) => backend.print_styled(&self.text, style),
                None => backend.print(&self.text),
            };
            backend.pad(max_width - self.width);
        } else {
            let mut remaining = self.width;
            let mut start = 0;
            match self.style.clone() {
                Some(style) => loop {
                    if remaining > max_width {
                        backend.print_styled(&self.text[start..start + max_width], style.clone());
                        remaining -= max_width;
                        start += max_width;
                    } else {
                        backend.print_styled(&self.text[start..], style.clone());
                        if max_width != remaining {
                            backend.pad(max_width - remaining);
                        }
                        return;
                    }
                    if lines.move_cursor(backend).is_none() {
                        return;
                    }
                },
                None => loop {
                    if remaining < max_width {
                        backend.print(&self.text[start..]);
                        if max_width != remaining {
                            backend.pad(max_width - remaining);
                        }
                    } else {
                        backend.print(&self.text[start..start + max_width]);
                        remaining -= max_width;
                        start += max_width;
                    }
                    if lines.move_cursor(backend).is_none() {
                        return;
                    }
                },
            }
        }
    }

    #[inline]
    fn wrap_with_remainder(&self, lines: &mut impl IterLines, backend: &mut B) -> Option<usize> {
        if self.is_simple() {
            self.wrap_with_remainder_simple(lines, backend)
        } else {
            self.wrap_with_remainder_complex(lines, backend)
        }
    }

    #[inline]
    pub fn wrap_with_remainder_simple(
        &self,
        lines: &mut impl IterLines,
        backend: &mut B,
    ) -> Option<usize> {
        let max_width = lines.move_cursor(backend)?;
        if max_width > self.width {
            match self.style.clone() {
                Some(style) => backend.print_styled(&self.text, style),
                None => backend.print(&self.text),
            };
            Some(max_width - self.width)
        } else {
            let mut remaining = self.width;
            let mut start = 0;
            match self.style.clone() {
                Some(style) => loop {
                    if remaining > max_width {
                        backend.print_styled(&self.text[start..start + max_width], style.clone());
                        remaining -= max_width;
                        start += max_width;
                    } else {
                        backend.print_styled(&self.text[start..], style.clone());
                        return Some(max_width - remaining);
                    }
                    lines.move_cursor(backend)?;
                },
                None => loop {
                    if remaining < max_width {
                        backend.print(&self.text[start..]);
                        return Some(max_width - remaining);
                    } else {
                        backend.print(&self.text[start..start + max_width]);
                        remaining -= max_width;
                        start += max_width;
                    }
                    lines.move_cursor(backend)?;
                },
            }
        }
    }

    #[inline]
    pub fn wrap_with_remainder_complex(
        &self,
        lines: &mut impl IterLines,
        backend: &mut B,
    ) -> Option<usize> {
        let max_width = lines.width();
        let mut chunks = WriteChunks::new(&self.text, max_width);
        let StrChunks {
            mut width,
            mut text,
        } = chunks.next()?;
        match self.style.clone() {
            Some(style) => loop {
                lines.move_cursor(backend)?;
                backend.print_styled(text, style.clone());
                match chunks.next() {
                    Some(next_chunk) => {
                        if width < max_width {
                            backend.pad(max_width - width);
                        }
                        StrChunks { width, text } = next_chunk;
                    }
                    None => {
                        return Some(max_width - width);
                    }
                }
            },
            None => loop {
                lines.move_cursor(backend)?;
                backend.print(text);
                match chunks.next() {
                    Some(next_chunk) => {
                        if width < max_width {
                            backend.pad(max_width - width);
                        }
                        StrChunks { width, text } = next_chunk;
                    }
                    None => {
                        return Some(max_width - width);
                    }
                }
            },
        }
    }
}

impl<B: Backend> Writable<B> for Text<B> {
    #[inline(always)]
    fn is_simple(&self) -> bool {
        self.char_len == self.text.len()
    }

    #[inline(always)]
    fn char_len(&self) -> usize {
        self.char_len
    }

    #[inline(always)]
    fn width(&self) -> usize {
        self.width
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.text.len()
    }

    fn print(&self, backend: &mut B) {
        match self.style.clone() {
            Some(style) => backend.print_styled(&self.text, style),
            None => backend.print(&self.text),
        }
    }

    unsafe fn print_truncated(&self, width: usize, backend: &mut B) {
        if self.is_simple() {
            match self.style.clone() {
                Some(style) => backend.print_styled(self.text.get_unchecked(..width), style),
                None => backend.print(self.text.get_unchecked(..width)),
            }
        } else {
            let (remaining_w, text) = self.text.truncate_width(width);
            match self.style.clone() {
                Some(style) => backend.print_styled(text, style),
                None => backend.print(text),
            }
            if remaining_w != 0 {
                backend.pad(remaining_w);
            }
        };
    }

    unsafe fn print_truncated_start(&self, width: usize, backend: &mut B) {
        if self.is_simple() {
            match self.style.clone() {
                Some(style) => {
                    backend.print_styled(self.text.get_unchecked(self.len() - width..), style)
                }
                None => backend.print(self.text.get_unchecked(self.len() - width..)),
            }
        } else {
            let (remaining_w, text) = self.text.truncate_width_start(width);
            if remaining_w != 0 {
                backend.pad(remaining_w);
            }
            match self.style.clone() {
                Some(style) => backend.print_styled(text, style),
                None => backend.print(text),
            }
        };
    }

    fn print_at(&self, line: Line, backend: &mut B) {
        let Line { width, row, col } = line;
        backend.go_to(row, col);
        if self.width > width {
            unsafe { self.print_truncated(width, backend) };
            return;
        }
        let pad_width = width - self.width;
        self.print(backend);
        if pad_width != 0 {
            backend.pad(pad_width);
        }
    }

    fn wrap(&self, lines: &mut impl IterLines, backend: &mut B) {
        match self.wrap_with_remainder(lines, backend) {
            Some(pad_width) if pad_width != 0 => backend.pad(pad_width),
            _ => (),
        }
    }
}

/// Collection of styled texts, useful when rendering multiple times the same string, as it holds meta data for width / charcer len of words
#[derive(Clone, PartialEq, Default)]
pub struct StyledLine<B: Backend> {
    inner: Vec<Text<B>>,
}

impl<B: Backend> Writable<B> for StyledLine<B> {
    fn is_simple(&self) -> bool {
        self.inner.iter().all(|text| text.is_simple())
    }

    #[inline(always)]
    fn char_len(&self) -> usize {
        self.inner.iter().fold(0, |sum, text| sum + text.char_len)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.iter().fold(0, |sum, text| sum + text.len())
    }

    fn width(&self) -> usize {
        self.inner.iter().fold(0, |sum, text| sum + text.width)
    }

    fn print(&self, backend: &mut B) {
        for text in self.inner.iter() {
            text.print(backend)
        }
    }

    unsafe fn print_truncated(&self, mut width: usize, backend: &mut B) {
        for text in self.inner.iter() {
            if text.width > width {
                text.print_truncated(width, backend);
                return;
            }
            width -= text.width;
            text.print(backend);
        }
    }

    unsafe fn print_truncated_start(&self, width: usize, backend: &mut B) {
        let mut skipped = self.width() - width;
        let mut iter = self.inner.iter();
        for text in iter.by_ref() {
            if text.width > skipped {
                text.print_truncated_start(text.width - skipped, backend);
                break;
            }
            skipped -= text.width;
        }

        for text in iter {
            text.print(backend);
        }
    }

    fn print_at(&self, line: Line, backend: &mut B) {
        let Line {
            row,
            col,
            mut width,
        } = line;
        backend.go_to(row, col);
        for text in self.inner.iter() {
            if width < text.width {
                unsafe { text.print_truncated(width, backend) };
                return;
            }
            width -= text.width;
            text.print(backend);
        }
        if width != 0 {
            backend.pad(width);
        }
    }

    fn wrap(&self, lines: &mut impl IterLines, backend: &mut B) {
        let mut width = match lines.move_cursor(backend) {
            Some(width) => width,
            None => return,
        };
        for word in self.inner.iter() {
            if word.width > width {
                if width == 0 {
                    width = match word.wrap_with_remainder(lines, backend) {
                        Some(new_width) => new_width,
                        None => return,
                    }
                } else if word.is_simple() {
                    let mut remaining = word.width;
                    let mut start = 0;
                    match word.style.clone() {
                        Some(style) => loop {
                            if remaining > width {
                                backend
                                    .print_styled(&word.text[start..start + width], style.clone());
                                remaining -= width;
                                start += width;
                            } else {
                                backend.print_styled(&word.text[start..], style.clone());
                                width -= remaining;
                                break;
                            }
                            match lines.move_cursor(backend) {
                                Some(max_width) => width = max_width,
                                None => return,
                            };
                        },
                        None => loop {
                            if remaining > width {
                                backend.print(&word.text[start..start + width]);
                                remaining -= width;
                                start += width;
                            } else {
                                backend.print(&word.text[start..]);
                                width -= remaining;
                                break;
                            }
                            match lines.move_cursor(backend) {
                                Some(max_width) => width = max_width,
                                None => return,
                            };
                        },
                    };
                } else {
                    match word.style.clone() {
                        Some(style) => {
                            for ch in word.text.chars() {
                                let ch_width = match UnicodeWidthChar::width(ch) {
                                    Some(ch_width) => ch_width,
                                    None => continue,
                                };
                                if ch_width > width {
                                    if width != 0 {
                                        backend.pad(width);
                                    }
                                    width = match lines.move_cursor(backend) {
                                        Some(new_width) => {
                                            backend.print_styled(ch, style.clone());
                                            new_width - ch_width
                                        }
                                        None => {
                                            if width != 0 {
                                                backend.pad(width);
                                            };
                                            return;
                                        }
                                    }
                                } else {
                                    backend.print_styled(ch, style.clone());
                                    width -= ch_width;
                                }
                            }
                        }
                        None => {
                            for ch in word.text.chars() {
                                let ch_width = match UnicodeWidthChar::width(ch) {
                                    Some(ch_width) => ch_width,
                                    None => continue,
                                };
                                if ch_width > width {
                                    if width != 0 {
                                        backend.pad(width);
                                    }
                                    width = match lines.move_cursor(backend) {
                                        Some(new_width) => {
                                            backend.print(ch);
                                            new_width - ch_width
                                        }
                                        None => {
                                            if width != 0 {
                                                backend.pad(width);
                                            };
                                            return;
                                        }
                                    }
                                } else {
                                    backend.print(ch);
                                    width -= ch_width;
                                }
                            }
                        }
                    }
                }
            } else {
                width -= word.width;
                word.print(backend);
            }
        }
        if width != 0 {
            backend.pad(width);
        }
    }
}

impl<B: Backend> Display for Text<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl<B: Backend> From<String> for Text<B> {
    fn from(text: String) -> Self {
        Self {
            char_len: text.char_len(),
            width: text.width(),
            text,
            style: None,
        }
    }
}

impl<B: Backend> From<char> for Text<B> {
    #[inline]
    fn from(value: char) -> Self {
        Self {
            char_len: 1,
            width: UnicodeWidthChar::width(value).unwrap_or_default(),
            text: value.to_string(),
            style: None,
        }
    }
}

impl<B: Backend> From<(String, <B as Backend>::Style)> for Text<B> {
    #[inline]
    fn from((text, style): (String, <B as Backend>::Style)) -> Self {
        Self {
            char_len: text.char_len(),
            width: text.width(),
            text,
            style: Some(style),
        }
    }
}

impl<B: Backend> Display for StyledLine<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for text in self.inner.iter() {
            text.fmt(f)?;
        }
        Ok(())
    }
}

impl<B: Backend> From<Vec<Text<B>>> for StyledLine<B> {
    fn from(inner: Vec<Text<B>>) -> Self {
        Self { inner }
    }
}

impl<B: Backend> From<String> for StyledLine<B> {
    fn from(text: String) -> Self {
        Self {
            inner: vec![text.into()],
        }
    }
}

impl<B: Backend> From<(String, <B as Backend>::Style)> for StyledLine<B> {
    fn from(text: (String, <B as Backend>::Style)) -> Self {
        Self {
            inner: vec![text.into()],
        }
    }
}

#[cfg(test)]
mod tests;
