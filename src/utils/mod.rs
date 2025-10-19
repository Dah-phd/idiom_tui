mod chunks;
pub use chunks::{ByteChunks, CharLimitedWidths, StrChunks, WriteChunks};
use std::ops::Range;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub type Utf8Byte = usize;
pub type Utf16Byte = usize;

/// Trait allowing UTF8 safe operations on str/String
pub trait UTFSafe {
    /// returns str that will fit into width of columns, removing chars at the end returning info about remaining width
    fn truncate_width(&self, width: usize) -> (usize, &str);
    /// returns str that will fit into width of columns, removing chars from the start returng info about remaining width
    fn truncate_width_start(&self, width: usize) -> (usize, &str);
    /// return Some(&str) if wider than allowed width
    fn truncate_if_wider(&self, width: usize) -> Result<&str, usize>;
    /// return Some(&str) truncated from start if wider than allowed width
    fn truncate_if_wider_start(&self, width: usize) -> Result<&str, usize>;
    /// split on width
    fn width_split(&self, width: usize) -> (&str, Option<&str>);
    /// returns display len of the str
    fn width(&self) -> usize;
    /// calcs the width at position
    fn width_at(&self, at: usize) -> usize;
    /// returns utf8 chars len
    fn char_len(&self) -> usize;
    /// utf16 len
    fn utf16_len(&self) -> usize;
    /// return utf8 split at char idx
    fn split_at_char(&self, mid: usize) -> (&str, &str);
    /// splits utf8 if not ascii (needs precalculated utf8 len)
    fn cached_split_at_char(&self, mid: usize, utf8_len: usize) -> (&str, &str);
    /// limits str within range based on utf char locations
    /// panics if out of bounds
    fn unchecked_get_char_range(&self, from: usize, to: usize) -> &str;
    /// removes "from" chars from the begining of the string
    /// panics if out of bounds
    fn unchecked_get_from_char(&self, from: usize) -> &str;
    /// limits str to char idx
    /// panics if out of bounds
    fn unchecked_get_to_char(&self, to: usize) -> &str;
    /// get checked utf8 slice
    fn get_char_range(&self, from_char: usize, to_char: usize) -> Option<&str>;
    /// get checked utf8 from
    fn get_from_char(&self, from_char: usize) -> Option<&str>;
    /// get checked utf8 to
    fn get_to_char(&self, to_char: usize) -> Option<&str>;
}

/// String specific extension
pub trait UTFSafeStringExt {
    fn insert_at_char(&mut self, idx: usize, ch: char);
    fn insert_at_char_with_utf8_idx(&mut self, idx: usize, ch: char) -> Utf8Byte;
    fn insert_at_char_with_utf16_idx(&mut self, idx: usize, ch: char) -> Utf16Byte;
    fn insert_str_at_char(&mut self, idx: usize, string: &str);
    fn insert_str_at_char_with_utf8_idx(&mut self, idx: usize, string: &str) -> Utf8Byte;
    fn insert_str_at_char_with_utf16_idx(&mut self, idx: usize, string: &str) -> Utf16Byte;
    fn remove_at_char(&mut self, idx: usize) -> char;
    /// returns the removed char with the utf8 idx from where it was removed
    fn remove_at_char_with_utf8_idx(&mut self, idx: usize) -> (Utf8Byte, char);
    /// returns the removed char with the utf16 idx from where it was removed
    fn remove_at_char_with_utf16_idx(&mut self, idx: usize) -> (Utf16Byte, char);
    fn replace_char_range(&mut self, range: Range<usize>, string: &str);
    fn replace_till_char(&mut self, to: usize, string: &str);
    fn replace_from_char(&mut self, from: usize, string: &str);
    fn split_off_at_char(&mut self, at: usize) -> Self;
}

impl UTFSafe for str {
    #[inline]
    fn truncate_width(&self, mut width: usize) -> (usize, &str) {
        let mut end = 0;
        for char in self.chars() {
            let char_width = UnicodeWidthChar::width(char).unwrap_or(0);
            if char_width > width {
                return (width, unsafe { self.get_unchecked(..end) });
            };
            width -= char_width;
            end += char.len_utf8();
        }
        (width, self)
    }

    #[inline]
    fn truncate_width_start(&self, mut width: usize) -> (usize, &str) {
        let mut start = 0;
        for char in self.chars().rev() {
            let char_width = UnicodeWidthChar::width(char).unwrap_or(0);
            if char_width > width {
                return (width, unsafe { self.get_unchecked(self.len() - start..) });
            }
            width -= char_width;
            start += char.len_utf8();
        }
        (width, self)
    }

    #[inline]
    fn truncate_if_wider(&self, width: usize) -> Result<&str, usize> {
        let mut end = 0;
        let mut current_width = 0;
        for char in self.chars() {
            current_width += UnicodeWidthChar::width(char).unwrap_or(0);
            if current_width > width {
                return Ok(unsafe { self.get_unchecked(..end) });
            };
            end += char.len_utf8();
        }
        Err(current_width)
    }

    #[inline]
    fn truncate_if_wider_start(&self, width: usize) -> Result<&str, usize> {
        let mut start = 0;
        let mut current_width = 0;
        for char in self.chars().rev() {
            current_width += UnicodeWidthChar::width(char).unwrap_or(0);
            if current_width > width {
                return Ok(unsafe { self.get_unchecked(self.len() - start..) });
            }
            start += char.len_utf8();
        }
        Err(current_width)
    }

    #[inline]
    fn width_split(&self, mut width: usize) -> (&str, Option<&str>) {
        for (current_mid, ch) in self.char_indices() {
            let ch_width = ch.width().unwrap_or(0);
            match ch_width > width {
                true => {
                    let (current, remaining) = self.split_at(current_mid);
                    return (current, Some(remaining));
                }
                false => {
                    width -= ch_width;
                }
            }
        }
        (self, None)
    }

    #[inline]
    fn width(&self) -> usize {
        UnicodeWidthStr::width(self)
    }

    #[inline]
    fn width_at(&self, at: usize) -> usize {
        self.chars()
            .take(at)
            .fold(0, |l, r| l + UnicodeWidthChar::width(r).unwrap_or(0))
    }

    #[inline]
    fn char_len(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn utf16_len(&self) -> usize {
        self.chars().fold(0, |sum, ch| sum + ch.len_utf16())
    }

    #[inline]
    fn split_at_char(&self, mid: usize) -> (&str, &str) {
        self.split_at(prev_char_bytes_end(self, mid))
    }

    #[inline]
    fn cached_split_at_char(&self, mid: usize, utf8_len: usize) -> (&str, &str) {
        if self.len() == utf8_len {
            return self.split_at(mid);
        }
        self.split_at_char(mid)
    }

    #[inline]
    fn get_char_range(&self, from: usize, to: usize) -> Option<&str> {
        maybe_prev_char_bytes_end(self, from)
            .and_then(|from_checked| Some(from_checked..maybe_prev_char_bytes_end(self, to)?))
            .map(|range| unsafe { self.get_unchecked(range) })
    }

    #[inline]
    fn get_from_char(&self, from: usize) -> Option<&str> {
        maybe_prev_char_bytes_end(self, from)
            .map(|from_checked| unsafe { self.get_unchecked(from_checked..) })
    }

    #[inline]
    fn get_to_char(&self, to: usize) -> Option<&str> {
        maybe_prev_char_bytes_end(self, to)
            .map(|to_checked| unsafe { self.get_unchecked(..to_checked) })
    }

    #[inline]
    fn unchecked_get_char_range(&self, from: usize, to: usize) -> &str {
        unsafe {
            self.get_unchecked(prev_char_bytes_end(self, from)..prev_char_bytes_end(self, to))
        }
    }

    #[inline]
    fn unchecked_get_from_char(&self, from: usize) -> &str {
        unsafe { self.get_unchecked(prev_char_bytes_end(self, from)..) }
    }

    #[inline]
    fn unchecked_get_to_char(&self, to: usize) -> &str {
        unsafe { self.get_unchecked(..prev_char_bytes_end(self, to)) }
    }
}

impl UTFSafe for String {
    #[inline]
    fn truncate_width(&self, width: usize) -> (usize, &str) {
        self.as_str().truncate_width(width)
    }

    #[inline]
    fn truncate_width_start(&self, width: usize) -> (usize, &str) {
        self.as_str().truncate_width_start(width)
    }

    #[inline]
    fn truncate_if_wider(&self, width: usize) -> Result<&str, usize> {
        self.as_str().truncate_if_wider(width)
    }

    #[inline]
    fn truncate_if_wider_start(&self, width: usize) -> Result<&str, usize> {
        self.as_str().truncate_if_wider_start(width)
    }

    #[inline]
    fn width_split(&self, width: usize) -> (&str, Option<&str>) {
        self.as_str().width_split(width)
    }

    #[inline]
    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.as_str())
    }

    #[inline]
    fn width_at(&self, at: usize) -> usize {
        self.as_str().width_at(at)
    }

    #[inline]
    fn char_len(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn utf16_len(&self) -> usize {
        self.as_str().utf16_len()
    }

    #[inline]
    fn split_at_char(&self, mid: usize) -> (&str, &str) {
        self.as_str().split_at_char(mid)
    }

    #[inline]
    fn cached_split_at_char(&self, mid: usize, utf8_len: usize) -> (&str, &str) {
        self.as_str().cached_split_at_char(mid, utf8_len)
    }

    #[inline]
    fn get_char_range(&self, from: usize, to: usize) -> Option<&str> {
        self.as_str().get_char_range(from, to)
    }

    #[inline]
    fn get_from_char(&self, from: usize) -> Option<&str> {
        self.as_str().get_from_char(from)
    }

    #[inline]
    fn get_to_char(&self, to: usize) -> Option<&str> {
        self.as_str().get_to_char(to)
    }

    #[inline]
    fn unchecked_get_char_range(&self, from: usize, to: usize) -> &str {
        self.as_str().unchecked_get_char_range(from, to)
    }

    #[inline]
    fn unchecked_get_from_char(&self, from: usize) -> &str {
        self.as_str().unchecked_get_from_char(from)
    }

    #[inline]
    fn unchecked_get_to_char(&self, to: usize) -> &str {
        self.as_str().unchecked_get_to_char(to)
    }
}

impl UTFSafeStringExt for String {
    #[inline]
    fn insert_at_char(&mut self, idx: usize, ch: char) {
        self.insert(prev_char_bytes_end(self, idx), ch);
    }

    #[inline]
    fn insert_at_char_with_utf8_idx(&mut self, idx: usize, ch: char) -> Utf8Byte {
        let byte_idx = prev_char_bytes_end(self, idx);
        self.insert(byte_idx, ch);
        byte_idx
    }

    #[inline]
    fn insert_at_char_with_utf16_idx(&mut self, idx: usize, ch: char) -> Utf16Byte {
        let (byte_idx, utf16_byte) = prev_char_utf8_and_utf16(self, idx);
        self.insert(byte_idx, ch);
        utf16_byte
    }

    #[inline]
    fn insert_str_at_char(&mut self, idx: usize, string: &str) {
        self.insert_str(prev_char_bytes_end(self, idx), string)
    }

    #[inline]
    fn insert_str_at_char_with_utf8_idx(&mut self, idx: usize, string: &str) -> Utf8Byte {
        let byte_idx = prev_char_bytes_end(self, idx);
        self.insert_str(byte_idx, string);
        byte_idx
    }

    #[inline]
    fn insert_str_at_char_with_utf16_idx(&mut self, idx: usize, string: &str) -> Utf16Byte {
        let (byte_idx, utf16_byte) = prev_char_utf8_and_utf16(self, idx);
        self.insert_str(byte_idx, string);
        utf16_byte
    }

    #[inline]
    fn remove_at_char(&mut self, idx: usize) -> char {
        self.remove(prev_char_bytes_end(self, idx))
    }

    #[inline]
    fn remove_at_char_with_utf8_idx(&mut self, idx: usize) -> (Utf8Byte, char) {
        let byte_idx = prev_char_bytes_end(self, idx);
        (byte_idx, self.remove(byte_idx))
    }

    #[inline]
    fn remove_at_char_with_utf16_idx(&mut self, idx: usize) -> (Utf16Byte, char) {
        let (byte_idx, utf16_byte) = prev_char_utf8_and_utf16(self, idx);
        (utf16_byte, self.remove(byte_idx))
    }

    #[inline]
    fn replace_char_range(&mut self, range: Range<usize>, text: &str) {
        let start = prev_char_bytes_end(self, range.start);
        let end = prev_char_bytes_end(self, range.end);
        self.replace_range(start..end, text);
    }

    #[inline]
    fn replace_from_char(&mut self, from: usize, string: &str) {
        self.truncate(prev_char_bytes_end(self, from));
        self.push_str(string);
    }

    #[inline]
    fn replace_till_char(&mut self, to: usize, string: &str) {
        self.replace_range(..prev_char_bytes_end(self, to), string);
    }

    #[inline]
    fn split_off_at_char(&mut self, at: usize) -> Self {
        self.split_off(prev_char_bytes_end(self, at))
    }
}

#[inline]
fn prev_char_bytes_end(text: &str, idx: usize) -> Utf8Byte {
    if idx == 0 {
        return 0;
    }
    if let Some((byte_idx, ch)) = text.char_indices().nth(idx - 1) {
        return byte_idx + ch.len_utf8();
    }
    panic!(
        "Index out of bound! Max len {} with index {}",
        text.char_len(),
        idx
    )
}

#[inline]
fn prev_char_utf8_and_utf16(text: &str, idx: usize) -> (Utf8Byte, Utf16Byte) {
    if idx == 0 {
        return (0, 0);
    }
    let mut counter = idx;
    let mut utf8_byte = 0;
    let mut utf16_byte = 0;
    for ch in text.chars() {
        counter -= 1;
        utf8_byte += ch.len_utf8();
        utf16_byte += ch.len_utf16();
        if counter == 0 {
            return (utf8_byte, utf16_byte);
        }
    }
    panic!(
        "Index out of bound! Max len {} with index {}",
        text.char_len(),
        idx
    )
}

#[inline]
fn maybe_prev_char_bytes_end(text: &str, idx: usize) -> Option<usize> {
    if idx == 0 {
        return Some(idx);
    }
    text.char_indices()
        .nth(idx - 1)
        .map(|(byte_idx, ch)| byte_idx + ch.len_utf8())
}

#[cfg(test)]
mod tests;
