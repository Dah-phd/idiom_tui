use super::{backend::Backend, UTF8Safe};
use core::ops::{Add, AddAssign, Range};
use unicode_width::UnicodeWidthChar;

#[cfg(feature = "crossterm_backend")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    count_as_string,
    layout::{Line, LineBuilder},
};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Status {
    #[default]
    Skipped,
    UpdatedCursor,
    Updated,
}

impl Status {
    /// includes cursor updates
    pub fn is_updated(&self) -> bool {
        match self {
            Self::Updated | Self::UpdatedCursor => true,
            Self::Skipped => false,
        }
    }

    pub fn is_text_updated(&self) -> bool {
        match self {
            Self::Updated => true,
            Self::UpdatedCursor | Self::Skipped => false,
        }
    }
}

impl Add for Status {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        std::cmp::max(self, rhs)
    }
}

impl AddAssign for Status {
    fn add_assign(&mut self, rhs: Self) {
        if &rhs > self {
            *self = rhs;
        }
    }
}

/// Single line input field
/// good for search boxes and filters
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

    pub fn cursor(&self) -> usize {
        self.char
    }

    pub fn select(&self) -> Option<(usize, usize)> {
        self.select
            .map(|(f, t)| if f > t { (t, f) } else { (f, t) })
    }

    pub fn select_drop(&mut self) -> Status {
        match self.select.take() {
            Some(..) => Status::UpdatedCursor,
            None => Status::Skipped,
        }
    }

    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn text_set(&mut self, text: String) {
        self.select = None;
        self.text = text;
        self.char = self.text.len();
    }

    pub fn cursor_set(&mut self, new_char: usize) -> Status {
        self.select_drop()
            + if self.text.len() < new_char {
                match self.char == self.text.len() {
                    true => Status::Skipped,
                    false => {
                        self.char = self.text.len();
                        Status::UpdatedCursor
                    }
                }
            } else if new_char == self.char {
                Status::Skipped
            } else {
                self.char = new_char;
                Status::UpdatedCursor
            }
    }

    pub fn text_take(&mut self) -> String {
        self.char = 0;
        self.select = None;
        std::mem::take(&mut self.text)
    }

    pub fn select_token_at_cursor(&mut self) -> Status {
        let token_range = arg_range_at(&self.text, self.char);
        if token_range.is_empty() {
            return Status::Skipped;
        }
        let new_select = Some((token_range.start, token_range.end));
        if self.select == new_select && self.char == token_range.end {
            return Status::Skipped;
        }
        self.select = new_select;
        self.char = token_range.end;
        Status::UpdatedCursor
    }

    pub fn get_token_at_cursor(&self) -> Option<&str> {
        let token_range = arg_range_at(&self.text, self.char);
        self.text.get(token_range)
    }

    pub fn replace_token(&mut self, new: &str) {
        let token_range = arg_range_at(&self.text, self.char);
        self.char = new.len() + token_range.start;
        self.select = None;
        self.text.replace_range(token_range, new);
    }

    // RENDER

    /// returns blockless paragraph widget " >> inner text"
    pub fn widget<B: Backend>(
        &self,
        line: Line,
        cursor_style: <B as Backend>::Style,
        select_style: <B as Backend>::Style,
        backend: &mut B,
    ) {
        let mut builder = line.unsafe_builder(backend);
        builder.push(" >> ");
        self.insert_formatted_text(builder, cursor_style, select_style);
    }

    /// returns blockless paragraph widget "99+ >> inner text"
    pub fn widget_with_count<B: Backend>(
        &self,
        line: Line,
        count: usize,
        cursor_style: <B as Backend>::Style,
        select_style: <B as Backend>::Style,
        backend: &mut B,
    ) {
        let mut builder = line.unsafe_builder(backend);
        builder.push(count_as_string(count).as_str());
        builder.push(" >> ");
        self.insert_formatted_text(builder, cursor_style, select_style);
    }

    pub fn insert_formatted_text<B: Backend>(
        &self,
        line_builder: LineBuilder<B>,
        cursor_style: <B as Backend>::Style,
        select_style: <B as Backend>::Style,
    ) {
        if line_builder.width() == 0 {
            return;
        }
        match self.select() {
            Some((from, to)) if from != to => {
                self.text_cursor_select(from, to, cursor_style, select_style, line_builder)
            }
            _ => self.text_cursor(cursor_style, line_builder),
        };
    }

    fn text_cursor<B: Backend>(
        &self,
        cursor_style: <B as Backend>::Style,
        mut builder: LineBuilder<B>,
    ) {
        let offset = self.calculate_width_offset(builder.width());
        match self.get_cursor_range() {
            Some(cursor) => {
                let Range { start, end } = cursor;
                builder.push(&self.text[offset..start]);
                builder.push_styled(&self.text[cursor], cursor_style);
                builder.push(&self.text[end..]);
            }
            None => {
                builder.push(&self.text[offset..]);
                builder.push_styled(" ", cursor_style);
            }
        }
    }

    fn text_cursor_select<B: Backend>(
        &self,
        mut from: usize,
        to: usize,
        cursor_style: <B as Backend>::Style,
        select_style: <B as Backend>::Style,
        mut builder: LineBuilder<B>,
    ) {
        let offset = self.calculate_width_offset(builder.width());
        if offset < from {
            builder.push(self.text[offset..from].as_ref());
        } else {
            from = offset;
        }
        match self.get_cursor_range() {
            Some(cursor) => {
                let Range { start, end } = cursor;
                if from == cursor.start {
                    builder.push_styled(&self.text[cursor], cursor_style);
                    builder.push_styled(&self.text[end..to], select_style);
                    builder.push(&self.text[to..]);
                } else {
                    builder.push_styled(&self.text[from..start], select_style);
                    builder.push_styled(&self.text[cursor], cursor_style);
                    builder.push(&self.text[end..]);
                }
            }
            None => {
                builder.push_styled(&self.text[from..], select_style);
                builder.push_styled(" ", cursor_style);
            }
        }
    }

    fn calculate_width_offset(&self, max_width: usize) -> usize {
        // in all cases byte index is greater than column width
        // so if avail width is bigger it is safe to skip offset
        // in most cases at least one char after cursor will be visible
        // in some using very strange chaars (over 3 cols - it could have visual artefacts)
        if self.char + 1 < max_width {
            return 0;
        }
        let cursor_prefix = &self.text[..self.char];
        let mut cursor_prefix_w = cursor_prefix.width() + 2;
        for (offset, ch) in cursor_prefix.char_indices() {
            if max_width > cursor_prefix_w {
                return offset;
            }
            if let Some(ch_width) = ch.width() {
                cursor_prefix_w = cursor_prefix_w.saturating_sub(ch_width);
            }
        }
        self.char
    }

    // CLIPBOARD LOGIC

    pub fn paste_passthrough(&mut self, clip: String) -> Status {
        if clip.contains('\n') {
            return Status::default();
        };
        self.take_selected();
        self.text.insert_str(self.char, clip.as_str());
        self.char += clip.len();
        Status::Updated
    }

    #[inline]
    pub fn copy(&mut self) -> Option<String> {
        self.get_selected()
    }

    #[inline]
    pub fn cut(&mut self) -> Option<String> {
        self.take_selected()
    }

    pub fn select_all(&mut self) -> Status {
        if self.text.is_empty() {
            return Status::Skipped;
        }
        let new_select = Some((0, self.text.len()));
        if self.char == self.text.len() && self.select == new_select {
            return Status::Skipped;
        }
        self.select = new_select;
        self.char = self.text.len();
        Status::UpdatedCursor
    }

    pub fn start_of_line(&mut self) -> Status {
        if self.char == 0 && self.select.is_none() {
            return Status::Skipped;
        }
        self.char = 0;
        self.select = None;
        Status::UpdatedCursor
    }

    pub fn end_of_line(&mut self) -> Status {
        if self.char == self.text.len() && self.select.is_none() {
            return Status::Skipped;
        }
        self.char = self.text.len();
        self.select = None;
        Status::UpdatedCursor
    }

    pub fn push_char(&mut self, ch: char) -> Status {
        self.take_selected();
        self.text.insert(self.char, ch);
        self.char += ch.len_utf8();
        Status::Updated
    }

    pub fn del(&mut self) -> Status {
        if self.take_selected().is_some() {
            Status::Updated
        } else if self.char < self.text.len() && !self.text.is_empty() {
            self.text.remove(self.char);
            Status::Updated
        } else {
            Status::Skipped
        }
    }

    pub fn backspace(&mut self) -> Status {
        if self.take_selected().is_some() {
            Status::Updated
        } else if self.char > 0 && !self.text.is_empty() {
            self.prev_char();
            self.text.remove(self.char);
            Status::Updated
        } else {
            Status::Skipped
        }
    }

    pub fn go_left(&mut self) -> Status {
        self.select_drop() + self.prev_char()
    }

    pub fn select_left(&mut self) -> Status {
        self.init_select() + self.prev_char() + self.push_select()
    }

    pub fn jump_left(&mut self) -> Status {
        self.select_drop() + self.prev_char() + self.jump_left_move()
    }

    pub fn select_jump_left(&mut self) -> Status {
        self.init_select() + self.prev_char() + self.jump_left_move() + self.push_select()
    }

    pub fn go_right(&mut self) -> Status {
        self.select_drop() + self.next_char()
    }

    pub fn select_right(&mut self) -> Status {
        self.init_select() + self.next_char() + self.push_select()
    }

    pub fn jump_right(&mut self) -> Status {
        self.select_drop() + self.next_char() + self.jump_right_move()
    }

    pub fn select_jump_right(&mut self) -> Status {
        self.init_select() + self.next_char() + self.jump_right_move() + self.push_select()
    }

    fn get_cursor_range(&self) -> Option<Range<usize>> {
        let cursor_char = self.text[self.char..].chars().next()?;
        Some(self.char..self.char + cursor_char.len_utf8())
    }

    fn next_char(&mut self) -> Status {
        match self.text[self.char..]
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
        {
            Some(offset) => {
                self.char += offset;
                Status::UpdatedCursor
            }
            None => Status::Skipped,
        }
    }

    fn prev_char(&mut self) -> Status {
        match self.text[..self.char]
            .chars()
            .next_back()
            .map(|ch| ch.len_utf8())
        {
            Some(offset) => {
                self.char -= offset;
                Status::UpdatedCursor
            }
            None => Status::Skipped,
        }
    }

    fn jump_left_move(&mut self) -> Status {
        let mut new_char = self.char;
        for (idx, ch) in self.text[..self.char].char_indices().rev() {
            if !should_jump(ch) {
                break;
            }
            new_char = idx;
        }
        if new_char == self.char {
            return Status::Skipped;
        }
        self.char = new_char;
        Status::UpdatedCursor
    }

    fn jump_right_move(&mut self) -> Status {
        for (idx, ch) in self.text[self.char..].char_indices() {
            if !should_jump(ch) {
                self.char += idx;
                return Status::UpdatedCursor;
            }
        }
        if self.char == self.text.len() {
            return Status::Skipped;
        }
        self.char = self.text.len();
        Status::UpdatedCursor
    }

    fn init_select(&mut self) -> Status {
        if self.select.is_some() {
            return Status::Skipped;
        }
        self.select = Some((self.char, self.char));
        Status::UpdatedCursor
    }

    fn push_select(&mut self) -> Status {
        match self.select.as_mut() {
            Some((_, to)) if to != &self.char => {
                *to = self.char;
                Status::UpdatedCursor
            }
            _ => Status::Skipped,
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

#[cfg(feature = "crossterm_backend")]
impl TextField {
    /// Maps crossterm key events
    /// if None is returned the key is not mapped at all
    /// Copy / Cut / Paste logic is not included -> use copy / cut / paste_passthrough instead
    pub fn map(&mut self, key: KeyEvent) -> Option<Status> {
        match key.code {
            KeyCode::Char('a' | 'A') if key.modifiers == KeyModifiers::CONTROL => {
                Some(self.select_all())
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(self.push_char(ch))
            }
            KeyCode::Delete => Some(self.del()),
            KeyCode::Backspace => Some(self.backspace()),
            KeyCode::Home => Some(self.start_of_line()),
            KeyCode::End => Some(self.end_of_line()),
            KeyCode::Left => Some(self.move_left(key.modifiers)),
            KeyCode::Right => Some(self.move_right(key.modifiers)),
            _ => None,
        }
    }

    fn move_left(&mut self, mods: KeyModifiers) -> Status {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        let mut status = if should_select {
            self.init_select()
        } else {
            self.select_drop()
        };
        status += self.prev_char();
        if mods.contains(KeyModifiers::CONTROL) {
            // jump
            status += self.jump_left_move();
        };
        if should_select {
            status += self.push_select();
        };
        status
    }

    fn move_right(&mut self, mods: KeyModifiers) -> Status {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        let mut status = if should_select {
            self.init_select()
        } else {
            self.select_drop()
        };
        status += self.next_char();
        if mods.contains(KeyModifiers::CONTROL) {
            // jump
            self.jump_right_move();
        };
        if should_select {
            status += self.push_select();
        };
        status
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

#[inline]
fn should_jump(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_numeric()
}

#[cfg(test)]
mod test {
    use crate::backend::{Backend, MockedBackend, MockedStyle};
    use crate::layout::Line;
    #[allow(unused)]
    use crate::text_field::Status;

    use super::{should_jump, TextField};

    #[cfg(feature = "crossterm_backend")]
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn render_non_ascii() {
        let mut field = TextField::new("a a🦀🦀ssd asd 🦀s".to_owned());
        let mut backend = MockedBackend::init();
        let line = Line {
            row: 0,
            col: 1,
            width: 50,
        };
        field.widget(
            line,
            MockedStyle::default(),
            MockedStyle::default(),
            &mut backend,
        );
        assert_eq!(
            backend.drain(),
            &[
                (MockedStyle::default(), "<<go to row: 0 col: 1>>".to_owned()),
                (MockedStyle::default(), " >> ".to_owned()),
                (MockedStyle::default(), "a a🦀🦀ssd asd 🦀s".to_owned()),
                (MockedStyle::default(), " ".to_owned()),
                (MockedStyle::default(), "<<padding: 27>>".to_owned()),
            ]
        );

        field.char = 0;
        field.go_right();
        field.go_right();
        field.go_right();

        let line = Line {
            row: 0,
            col: 1,
            width: 50,
        };
        field.widget(
            line,
            MockedStyle::default(),
            MockedStyle::default(),
            &mut backend,
        );
        assert_eq!(
            backend.drain(),
            &[
                (MockedStyle::default(), "<<go to row: 0 col: 1>>".to_owned()),
                (MockedStyle::default(), " >> ".to_owned()),
                (MockedStyle::default(), "a a".to_owned()),
                (MockedStyle::default(), "🦀".to_owned()),
                (MockedStyle::default(), "🦀ssd asd 🦀s".to_owned()),
                (MockedStyle::default(), "<<padding: 28>>".to_owned()),
            ]
        );

        field.go_right();
        field.select_jump_right();

        let line = Line {
            row: 0,
            col: 1,
            width: 50,
        };
        field.widget(
            line,
            MockedStyle::default(),
            MockedStyle::default(),
            &mut backend,
        );
        assert_eq!(
            backend.drain(),
            &[
                (MockedStyle::default(), "<<go to row: 0 col: 1>>".to_owned()),
                (MockedStyle::default(), " >> ".to_owned()),
                (MockedStyle::default(), "a a🦀".to_owned()),
                (MockedStyle::default(), "🦀ssd".to_owned()),
                (MockedStyle::default(), " ".to_owned()),
                (MockedStyle::default(), "asd 🦀s".to_owned()),
                (MockedStyle::default(), "<<padding: 28>>".to_owned()),
            ]
        );
    }

    #[test]
    fn render_with_number() {
        let field = TextField::new("some text".to_owned());
        let mut backend = MockedBackend::init();
        let line = Line {
            row: 0,
            col: 1,
            width: 50,
        };

        field.widget_with_count(
            line,
            3,
            MockedStyle::default(),
            MockedStyle::default(),
            &mut backend,
        );

        assert_eq!(
            backend.drain(),
            &[
                (MockedStyle::default(), "<<go to row: 0 col: 1>>".to_owned()),
                (MockedStyle::default(), "  3".to_owned()),
                (MockedStyle::default(), " >> ".to_owned()),
                (MockedStyle::default(), "some text".to_owned()),
                (MockedStyle::default(), " ".to_owned()),
                (MockedStyle::default(), "<<padding: 33>>".to_owned()),
            ]
        );
    }

    #[test]
    fn test_should_jump() {
        assert!(should_jump('a'));
        assert!(should_jump('1'));
        assert!(should_jump('b'));
        assert!(!should_jump('🦀'));
    }

    #[test]
    fn get_select() {
        let mut t = TextField::default();
        t.select = Some((10, 5));
        assert_eq!(t.select().unwrap(), (5, 10));
        t.select = Some((3, 8));
        assert_eq!(t.select().unwrap(), (3, 8));
    }

    #[test]
    fn move_status() {
        let mut t = TextField::new("rand_text".into());
        assert_eq!(t.char, t.as_str().len());
        assert!(!t.go_right().is_updated());
        assert!(!t.jump_right().is_updated());
        assert!(t.select_right().is_updated());
        assert!(!t.select_right().is_updated());
        assert!(!t.select_jump_right().is_updated());

        assert!(t.go_left().is_updated());
        assert_eq!(t.char, 8);
        assert!(t.jump_left().is_updated());
        assert_eq!(t.char, 5);
        assert!(t.select().is_none());
        assert!(t.select_jump_left().is_updated());
        assert_eq!(t.char, 0);
        assert!(t.select().is_some());

        assert!(t.go_left().is_updated());
        assert!(t.select().is_none());
        assert!(!t.go_left().is_updated());
        assert!(!t.jump_left().is_updated());
        assert!(t.select_left().is_updated());
        assert!(!t.select_left().is_updated());
        assert!(!t.select_jump_left().is_updated());

        assert!(t.go_right().is_updated());
        assert_eq!(t.char, 1);
        assert!(t.jump_right().is_updated());
        assert_eq!(t.char, 4);
        assert!(t.select().is_none());
        assert!(t.select_jump_right().is_updated());
        assert_eq!(t.char, 9);
        assert!(t.select().is_some());
        assert!(t.go_right().is_updated());
        assert!(t.select().is_none());
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_setting() {
        let mut field = TextField::default();
        field.text_set("12345".to_owned());
        assert_eq!(&field.text, "12345");
        assert_eq!(field.char, 5);
        field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        assert!(field.select.is_some());
        assert_eq!(field.char, 4);
        assert_eq!(&field.text_take(), "12345");
        assert_eq!(field.char, 0);
        assert_eq!(&field.text, "");
        assert!(field.select.is_none());
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_setting_non_ascii() {
        let mut field = TextField::default();
        field.text_set("1234🦀".to_owned());
        assert_eq!(&field.text, "1234🦀");
        assert_eq!(field.char, 8);
        field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        assert_eq!(field.char, 4);
        field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        assert_eq!(field.char, 8);
        field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        assert!(field.select.is_some());
        assert_eq!(field.char, 4);
        assert_eq!(&field.text_take(), "1234🦀");
        assert_eq!(field.char, 0);
        assert_eq!(&field.text, "");
        assert!(field.select.is_none());
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_move() {
        let mut field = TextField::default();
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::Skipped)
        );
        assert!(field.char == 0);
        field.text_set("12".to_owned());
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Some(Status::Skipped)
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 0);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 0);
    }

    #[test]
    fn text_get_token_at_cursor() {
        let mut field = TextField::new("a a🦀sd xx".to_owned());
        field.char = 0;
        field.go_right();
        field.go_right();
        assert_eq!(field.char, 2);
        assert_eq!(field.get_token_at_cursor(), Some("a🦀sd"));
        let mut field = TextField::new("a asd xx".to_owned());
        field.char = 0;
        field.go_right();
        field.go_right();
        assert_eq!(field.char, 2);
        assert_eq!(field.get_token_at_cursor(), Some("asd"));
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_backspace() {
        let mut field = TextField::new("a a🦀🦀ssd".to_owned());
        field.go_left();
        assert_eq!(field.char, 13);
        assert_eq!(field.len(), 14);
        field.backspace();
        assert_eq!(field.char, 12);
        assert_eq!(field.as_str(), "a a🦀🦀sd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Some(Status::Updated)
        );
        assert_eq!(field.char, 11);
        assert_eq!(field.as_str(), "a a🦀🦀d");
        field.backspace();
        assert_eq!(field.char, 7);
        assert_eq!(field.as_str(), "a a🦀d");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Some(Status::Updated)
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a ad");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_del() {
        let mut field = TextField::new("a a🦀🦀ssd".to_owned());
        field.go_left();
        field.go_left();
        field.go_left();
        field.go_left();
        assert_eq!(field.char, 7);
        field.go_left();
        assert_eq!(field.char, 3);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Delete, KeyModifiers::empty())),
            Some(Status::Updated)
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a a🦀ssd");
        field.del();
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a assd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Delete, KeyModifiers::empty())),
            Some(Status::Updated)
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a asd");
        field.del();
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a ad");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn select() {
        let mut field = TextField::new("a axxssd as".to_owned());
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 9);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 8);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "axxssd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "xxssd");

        field.char = 0;
        field.select = None;
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "axxssd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 7);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "axxss");
    }

    #[test]
    fn select_calls() {
        let mut field = TextField::new("a axxssd as".to_owned());
        field.jump_left();
        assert_eq!(field.char, 9);
        field.go_left();
        assert_eq!(field.char, 8);
        field.select_jump_left();
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "axxssd");
        field.go_right();
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        field.select_jump_right();
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "xxssd");

        field.char = 0;
        field.select = None;
        field.jump_right();
        assert_eq!(field.char, 1);
        field.go_right();
        assert_eq!(field.char, 2);
        field.select_jump_right();
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "axxssd");
        field.go_left();
        assert_eq!(field.char, 7);
        assert_eq!(field.copy(), None);
        field.select_jump_left();
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "axxss");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn select_non_ascii() {
        let mut field = TextField::new("a a🦀🦀ssd a".to_owned());
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 15);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 14);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 11);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "🦀🦀ssd");

        field.char = 0;
        field.select = None;
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 3);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 13);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 7);
        assert_eq!(field.copy().unwrap(), "🦀ss");
    }

    #[test]
    fn select_non_ascii_calls() {
        let mut field = TextField::new("a a🦀🦀ssd a".to_owned());
        field.jump_left();
        assert_eq!(field.char, 15);
        field.go_left();
        assert_eq!(field.char, 14);
        field.select_jump_left();
        assert_eq!(field.char, 11);
        field.select_jump_left();
        field.select_jump_left();
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        field.go_right();
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        field.select_jump_right();
        field.select_jump_right();
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "🦀🦀ssd");

        field.char = 0;
        field.select = None;
        field.jump_right();
        assert_eq!(field.char, 1);
        field.go_right();
        assert_eq!(field.char, 2);
        field.select_jump_right();
        assert_eq!(field.char, 3);
        field.select_jump_right();
        field.select_jump_right();
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        field.go_left();
        assert_eq!(field.char, 13);
        assert_eq!(field.copy(), None);
        field.select_jump_left();
        field.select_jump_left();
        assert_eq!(field.char, 7);
        assert_eq!(field.copy().unwrap(), "🦀ss");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_move_non_ascii() {
        let mut field = TextField::default();
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::Skipped)
        );
        assert!(field.char == 0);
        field.text_set("1🦀2".to_owned());
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Some(Status::Skipped)
        );
        assert_eq!(field.char, 6);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 5);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 0);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 6);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 5);
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_select() {
        let mut field = TextField::default();
        field.text_set("a3cde".to_owned());
        field.char = 0;
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.select, Some((0, 5)));
        assert_eq!(field.char, 5);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert!(field.select.is_none());
        assert_eq!(field.char, 5);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.select, Some((5, 4)));
        assert_eq!(
            field.map(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.select, Some((5, 0)));
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Some(Status::Updated)
        );
        assert_eq!(&field.text, "");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_select_all_map() {
        let mut field = TextField::new("data".into());
        assert!(field.select.is_none());
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL,)),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 4);
        assert_eq!(field.get_selected().unwrap(), "data");
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_start_of_line() {
        let mut field = TextField::new("data".into());
        field.select_all();
        assert_eq!(field.get_selected().unwrap(), "data");
        assert_eq!(field.char, 4);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::Home, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 0);
        assert!(field.get_selected().is_none());
    }

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_end_of_line() {
        let mut field = TextField::new("data".into());
        field.select_jump_left();
        assert_eq!(field.get_selected().unwrap(), "data");
        assert_eq!(field.char, 0);
        assert_eq!(
            field.map(KeyEvent::new(KeyCode::End, KeyModifiers::empty())),
            Some(Status::UpdatedCursor)
        );
        assert_eq!(field.char, 4);
        assert!(field.get_selected().is_none());
    }

    #[test]
    fn test_select_all() {
        let mut field = TextField::new("data".into());
        assert!(field.select.is_none());
        field.select_all();
        assert_eq!(field.char, 4);
        assert_eq!(field.get_selected().unwrap(), "data");
    }

    #[test]
    fn test_ord_status() {
        assert!(Status::Skipped < Status::UpdatedCursor);
        assert!(Status::UpdatedCursor < Status::Updated);
        assert!(Status::Updated > Status::Skipped);
        assert!(Status::Updated == Status::Updated);
    }
}
