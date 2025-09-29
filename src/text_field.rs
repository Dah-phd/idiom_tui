use super::backend::{Backend, StyleExt};
use core::ops::Range;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, ContentStyle};

use super::{
    count_as_string,
    layout::{Line, LineBuilder},
};

#[derive(Default, PartialEq, Debug)]
pub enum Status {
    #[default]
    Skipped,
    Updated,
    UpdatedCursor,
    NotMapped,
    PasteInvoked,
    Copy(String),
    Cut(String),
}

impl Status {
    /// includes cursor updates
    pub fn is_updated(&self) -> bool {
        match self {
            Self::Updated | Self::UpdatedCursor | Self::Cut(..) => true,
            Self::Skipped | Self::NotMapped | Self::Copy(..) | Self::PasteInvoked => false,
        }
    }

    pub fn is_text_updated(&self) -> bool {
        match self {
            Self::Updated | Self::Cut(..) => true,
            Self::UpdatedCursor
            | Self::Skipped
            | Self::NotMapped
            | Self::Copy(..)
            | Self::PasteInvoked => false,
        }
    }

    pub fn is_mapped(&self) -> bool {
        !matches!(self, Self::NotMapped)
    }
}

#[derive(Default)]
pub struct TextField {
    text: String,
    char: usize,
    select: Option<(usize, usize)>,
}

impl TextField {
    pub fn new(text: String) -> Self {
        Self {
            char: text.len(),
            text,
            select: None,
        }
    }

    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn text_set(&mut self, text: String) {
        self.select = None;
        self.text = text;
        self.char = self.text.len();
    }

    pub fn text_take(&mut self) -> String {
        self.char = 0;
        self.select = None;
        std::mem::take(&mut self.text)
    }

    pub fn text_get_token_at_cursor(&self) -> Option<&str> {
        let token_range = arg_range_at(&self.text, self.char);
        self.text.get(token_range)
    }

    pub fn text_replace_token(&mut self, new: &str) {
        let token_range = arg_range_at(&self.text, self.char);
        self.char = new.len() + token_range.start;
        self.select = None;
        self.text.replace_range(token_range, new);
    }

    /// returns blockless paragraph widget " >> inner text"
    pub fn widget<B: Backend<Style = ContentStyle, Color = Color>>(
        &self,
        line: Line,
        backend: &mut B,
    ) {
        let mut builder = line.unsafe_builder(backend);
        builder.push(" >> ");
        self.insert_formatted_text(builder);
    }

    /// returns blockless paragraph widget "99+ >> inner text"
    pub fn widget_with_count<B: Backend<Style = ContentStyle, Color = Color>>(
        &self,
        line: Line,
        count: usize,
        backend: &mut B,
    ) {
        let mut builder = line.unsafe_builder(backend);
        builder.push(count_as_string(count).as_str());
        builder.push(" >> ");
        self.insert_formatted_text(builder);
    }

    pub fn insert_formatted_text<B: Backend<Style = ContentStyle, Color = Color>>(
        &self,
        line_builder: LineBuilder<B>,
    ) {
        match self
            .select
            .as_ref()
            .map(|(f, t)| if f > t { (*t, *f) } else { (*f, *t) })
        {
            Some((from, to)) if from != to => self.text_cursor_select(from, to, line_builder),
            _ => self.text_cursor(line_builder),
        };
    }

    fn text_cursor<B: Backend<Style = ContentStyle, Color = Color>>(
        &self,
        mut builder: LineBuilder<B>,
    ) {
        match self.get_cursor_range() {
            Some(cursor) => {
                let Range { start, end } = cursor;
                builder.push(&self.text[..start]);
                builder.push_styled(&self.text[cursor], ContentStyle::reversed());
                builder.push(&self.text[end..]);
            }
            None => {
                builder.push(&self.text);
                builder.push_styled(" ", ContentStyle::reversed());
            }
        }
        if self.char == self.text.len() {
        } else {
        };
    }

    fn text_cursor_select<B: Backend<Style = ContentStyle, Color = Color>>(
        &self,
        from: usize,
        to: usize,
        mut builder: LineBuilder<B>,
    ) {
        builder.push(self.text[..from].as_ref());
        let select_style = ContentStyle::bg(Color::Rgb {
            r: 72,
            g: 72,
            b: 72,
        });
        match self.get_cursor_range() {
            Some(cursor) => {
                let Range { start, end } = cursor;
                if from == cursor.start {
                    builder.push_styled(&self.text[cursor], ContentStyle::reversed());
                    builder.push_styled(&self.text[end..to], select_style);
                    builder.push(&self.text[to..]);
                } else {
                    builder.push_styled(&self.text[from..start], select_style);
                    builder.push_styled(&self.text[cursor], ContentStyle::reversed());
                    builder.push(&self.text[end..]);
                }
            }
            None => {
                builder.push_styled(self.text[from..to].as_ref(), select_style);
                builder.push(self.text[to..].as_ref());
                builder.push_styled(" ", ContentStyle::reversed());
            }
        }
    }

    fn get_cursor_range(&self) -> Option<Range<usize>> {
        let cursor_char = self.text[self.char..].chars().next()?;
        Some(self.char..self.char + cursor_char.len_utf8())
    }

    fn next_char(&mut self) {
        self.char += self.text[self.char..]
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or_default();
    }

    fn prev_char(&mut self) {
        self.char -= self.text[..self.char]
            .chars()
            .rev()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or_default();
    }

    pub fn paste_passthrough(&mut self, clip: String) -> Status {
        if !clip.contains('\n') {
            self.take_selected();
            self.text.insert_str(self.char, clip.as_str());
            self.char += clip.len();
            return Status::Updated;
        };
        Status::default()
    }

    pub fn map(&mut self, key: &KeyEvent) -> Status {
        match key.code {
            KeyCode::Char('c' | 'C')
                if key.modifiers == KeyModifiers::CONTROL
                    || key.modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT =>
            {
                if let Some(clip) = self.get_selected() {
                    return Status::Copy(clip);
                };
                Status::default()
            }
            KeyCode::Char('x' | 'X') if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(clip) = self.take_selected() {
                    return Status::Cut(clip);
                };
                Status::default()
            }
            KeyCode::Char('v' | 'V') if key.modifiers == KeyModifiers::CONTROL => {
                Status::PasteInvoked
            }
            KeyCode::Char(ch) => {
                self.take_selected();
                self.text.insert(self.char, ch);
                self.char += ch.len_utf8();
                Status::Updated
            }
            KeyCode::Delete => {
                if self.take_selected().is_some() {
                    return Status::Updated;
                };
                if self.char < self.text.len() && !self.text.is_empty() {
                    self.text.remove(self.char);
                    return Status::Updated;
                }
                Status::Skipped
            }
            KeyCode::Backspace => {
                if self.take_selected().is_some() {
                    return Status::Updated;
                };
                if self.char > 0 && !self.text.is_empty() {
                    self.prev_char();
                    self.text.remove(self.char);
                    return Status::Updated;
                };
                Status::Skipped
            }
            KeyCode::End => {
                self.char = self.text.len();
                Status::UpdatedCursor
            }
            KeyCode::Left => {
                self.move_left(key.modifiers);
                Status::UpdatedCursor
            }
            KeyCode::Right => {
                self.move_right(key.modifiers);
                Status::UpdatedCursor
            }
            _ => Status::NotMapped,
        }
    }

    pub fn copy(&mut self) -> Option<String> {
        self.get_selected()
    }

    pub fn cut(&mut self) -> Option<String> {
        self.take_selected()
    }

    /// returns false if clip contains new line
    pub fn try_paste(&mut self, clip: String) -> bool {
        if clip.contains('\n') {
            return false;
        }
        self.take_selected();
        self.text.insert_str(self.char, clip.as_str());
        self.char += clip.len();
        true
    }

    pub fn push_char(&mut self, ch: char) {
        self.take_selected();
        self.text.insert(self.char, ch);
        self.char += ch.len_utf8();
    }

    pub fn del(&mut self) {
        if self.take_selected().is_some() {
            return;
        }
        if self.char < self.text.len() && !self.text.is_empty() {
            self.text.remove(self.char);
        };
    }

    pub fn backspace(&mut self) {
        if self.take_selected().is_some() {
            return;
        }
        if self.char > 0 && !self.text.is_empty() {
            self.prev_char();
            self.text.remove(self.char);
        };
    }

    pub fn go_to_end_of_line(&mut self) {
        self.char = self.text.len();
    }

    pub fn go_left(&mut self) {
        self.select = None;
        self.prev_char();
    }

    pub fn select_left(&mut self) {
        self.init_select();
        self.prev_char();
        self.push_select();
    }

    pub fn jump_left(&mut self) {
        self.select = None;
        self.jump_right_move();
    }

    pub fn select_jump_left(&mut self) {
        self.init_select();
        self.jump_left_move();
        self.push_select();
    }

    pub fn go_right(&mut self) {
        self.select = None;
        self.next_char();
    }

    pub fn select_right(&mut self) {
        self.init_select();
        self.next_char();
        self.push_select();
    }

    pub fn jump_right(&mut self) {
        self.select = None;
        self.jump_left_move();
    }

    pub fn select_jump_right(&mut self) {
        self.init_select();
        self.jump_right_move();
        self.push_select();
    }

    fn move_left(&mut self, mods: KeyModifiers) {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        if should_select {
            self.init_select();
        } else {
            self.select = None;
        };
        self.char = self.char.saturating_sub(1);
        if mods.contains(KeyModifiers::CONTROL) {
            // jump
            self.jump_left_move();
        };
        if should_select {
            self.push_select();
        };
    }

    fn move_right(&mut self, mods: KeyModifiers) {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        if should_select {
            self.init_select();
        } else {
            self.select = None;
        };
        self.char = std::cmp::min(self.text.len(), self.char + 1);
        if mods.contains(KeyModifiers::CONTROL) {
            // jump
            self.jump_right_move();
        };
        if should_select {
            self.push_select();
        };
    }

    fn jump_left_move(&mut self) {
        for (idx, ch) in self.text[..self.char].char_indices().rev() {
            if !ch.is_alphabetic() && !ch.is_numeric() {
                return;
            }
            self.char = idx;
        }
    }

    fn jump_right_move(&mut self) {
        for (idx, ch) in self.text[self.char..].char_indices() {
            if !ch.is_alphabetic() && !ch.is_numeric() {
                self.char += idx;
                return;
            }
        }
        self.char = self.text.len();
    }

    fn init_select(&mut self) {
        if self.select.is_none() {
            self.select = Some((self.char, self.char))
        }
    }

    fn push_select(&mut self) {
        if let Some((_, to)) = self.select.as_mut() {
            *to = self.char;
        }
    }

    fn get_selected(&mut self) -> Option<String> {
        let (from, to) = self
            .select
            .map(|(f, t)| if f > t { (t, f) } else { (f, t) })?;
        if from == to {
            return None;
        }
        Some(self.text[from..to].to_owned())
    }

    fn take_selected(&mut self) -> Option<String> {
        let (from, to) = self
            .select
            .take()
            .map(|(f, t)| if f > t { (t, f) } else { (f, t) })?;
        if from == to {
            return None;
        }
        let clip = self.text[from..to].to_owned();
        self.text.replace_range(from..to, "");
        self.char = from;
        Some(clip)
    }
}

pub fn arg_range_at(line: &str, idx: usize) -> Range<usize> {
    let mut token_start = 0;
    let mut last_not_in_token = false;
    for (char_idx, ch) in line.char_indices() {
        if !ch.is_whitespace() {
            if last_not_in_token {
                token_start = char_idx;
            }
            last_not_in_token = false;
        } else if char_idx >= idx {
            if last_not_in_token {
                return idx..idx;
            }
            return token_start..char_idx;
        } else {
            last_not_in_token = true;
        }
    }
    if idx < line.len() {
        token_start..line.len()
    } else if !last_not_in_token && token_start <= idx {
        token_start..idx
    } else {
        idx..idx
    }
}

#[cfg(test)]
mod test {
    use crate::text_field::Status;

    use super::TextField;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_setting() {
        let mut field = TextField::default();
        field.text_set("12345".to_owned());
        assert_eq!(&field.text, "12345");
        assert_eq!(field.char, 5);
        field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        assert!(field.select.is_some());
        assert_eq!(field.char, 4);
        assert_eq!(&field.text_take(), "12345");
        assert_eq!(field.char, 0);
        assert_eq!(&field.text, "");
        assert!(field.select.is_none());
    }

    #[test]
    fn test_move() {
        let mut field = TextField::default();
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert!(field.char == 0);
        field.text_set("12".to_owned());
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 0);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 0);
    }

    #[test]
    fn test_select() {
        let mut field = TextField::default();
        field.text_set("a3cde".to_owned());
        field.char = 0;
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.select, Some((0, 5)));
        assert_eq!(field.char, 5);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert!(field.select.is_none());
        assert_eq!(field.char, 5);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)),
            Status::UpdatedCursor
        );
        assert_eq!(field.select, Some((5, 4)));
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.select, Some((5, 0)));
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Status::Updated
        );
        assert_eq!(&field.text, "");
    }
}
