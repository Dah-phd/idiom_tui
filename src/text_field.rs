use super::backend::Backend;
use core::ops::Range;

#[cfg(feature = "crossterm_backend")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
        match self
            .select
            .as_ref()
            .map(|(f, t)| if f > t { (*t, *f) } else { (*f, *t) })
        {
            Some((from, to)) if from != to => {
                self.text_cursor_select(from, to, cursor_style, select_style, line_builder)
            }
            _ => self.text_cursor(cursor_style, line_builder),
        };
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

    pub fn select_all(&mut self) {
        if self.text.is_empty() {
            return;
        }
        self.select = Some((0, self.text.len()));
        self.char = self.text.len();
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
        self.prev_char();
        self.jump_left_move();
    }

    pub fn select_jump_left(&mut self) {
        self.init_select();
        self.prev_char();
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
        self.next_char();
        self.jump_right_move();
    }

    pub fn select_jump_right(&mut self) {
        self.init_select();
        self.next_char();
        self.jump_right_move();
        self.push_select();
    }

    #[cfg(feature = "crossterm_backend")]
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

    fn text_cursor<B: Backend>(
        &self,
        cursor_style: <B as Backend>::Style,
        mut builder: LineBuilder<B>,
    ) {
        match self.get_cursor_range() {
            Some(cursor) => {
                let Range { start, end } = cursor;
                builder.push(&self.text[..start]);
                builder.push_styled(&self.text[cursor], cursor_style);
                builder.push(&self.text[end..]);
            }
            None => {
                builder.push(&self.text);
                builder.push_styled(" ", cursor_style);
            }
        }
    }

    fn text_cursor_select<B: Backend>(
        &self,
        from: usize,
        to: usize,
        cursor_style: <B as Backend>::Style,
        select_style: <B as Backend>::Style,
        mut builder: LineBuilder<B>,
    ) {
        builder.push(self.text[..from].as_ref());
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
                builder.push_styled(self.text[from..to].as_ref(), select_style);
                builder.push(self.text[to..].as_ref());
                builder.push_styled(" ", cursor_style);
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
            .next_back()
            .map(|ch| ch.len_utf8())
            .unwrap_or_default();
    }

    #[cfg(feature = "crossterm_backend")]
    fn move_left(&mut self, mods: KeyModifiers) {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        if should_select {
            self.init_select();
        } else {
            self.select = None;
        };
        self.prev_char();
        if mods.contains(KeyModifiers::CONTROL) {
            // jump
            self.jump_left_move();
        };
        if should_select {
            self.push_select();
        };
    }

    #[cfg(feature = "crossterm_backend")]
    fn move_right(&mut self, mods: KeyModifiers) {
        let should_select = mods.contains(KeyModifiers::SHIFT);
        if should_select {
            self.init_select();
        } else {
            self.select = None;
        };
        self.next_char();
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
            if !should_jump(ch) {
                return;
            }
            self.char = idx;
        }
    }

    fn jump_right_move(&mut self) {
        for (idx, ch) in self.text[self.char..].char_indices() {
            if !should_jump(ch) {
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

    #[cfg(feature = "crossterm_backend")]
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

    #[cfg(feature = "crossterm_backend")]
    #[test]
    fn test_setting_non_ascii() {
        let mut field = TextField::default();
        field.text_set("1234🦀".to_owned());
        assert_eq!(&field.text, "1234🦀");
        assert_eq!(field.char, 8);
        field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        assert_eq!(field.char, 4);
        field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        assert_eq!(field.char, 8);
        field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
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
    fn text_get_token_at_cursor() {
        let mut field = TextField::new("a a🦀sd xx".to_owned());
        field.char = 0;
        field.go_right();
        field.go_right();
        assert_eq!(field.char, 2);
        assert_eq!(field.text_get_token_at_cursor(), Some("a🦀sd"));
        let mut field = TextField::new("a asd xx".to_owned());
        field.char = 0;
        field.go_right();
        field.go_right();
        assert_eq!(field.char, 2);
        assert_eq!(field.text_get_token_at_cursor(), Some("asd"));
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
            field.map(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Status::Updated
        );
        assert_eq!(field.char, 11);
        assert_eq!(field.as_str(), "a a🦀🦀d");
        field.backspace();
        assert_eq!(field.char, 7);
        assert_eq!(field.as_str(), "a a🦀d");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())),
            Status::Updated
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
            field.map(&KeyEvent::new(KeyCode::Delete, KeyModifiers::empty())),
            Status::Updated
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a a🦀ssd");
        field.del();
        assert_eq!(field.char, 3);
        assert_eq!(field.as_str(), "a assd");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Delete, KeyModifiers::empty())),
            Status::Updated
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
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 9);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 8);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "axxssd");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "xxssd");

        field.char = 0;
        field.select = None;
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 8);
        assert_eq!(field.copy().unwrap(), "axxssd");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 7);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
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
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 15);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 14);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 11);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 2);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 3);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "🦀🦀ssd");

        field.char = 0;
        field.select = None;
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 1);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 2);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 3);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 14);
        assert_eq!(field.copy().unwrap(), "a🦀🦀ssd");
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 13);
        assert_eq!(field.copy(), None);
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
        );
        assert_eq!(
            field.map(&KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Status::UpdatedCursor
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
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
            Status::UpdatedCursor
        );
        assert!(field.char == 0);
        field.text_set("1🦀2".to_owned());
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 6);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 5);
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
            field.map(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            Status::UpdatedCursor
        );
        assert_eq!(field.char, 6);
        assert_eq!(
            field.map(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
            Status::UpdatedCursor
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

    #[test]
    fn test_select_all() {
        let mut field = TextField::new("data".into());
        assert!(field.select.is_none());
        field.select_all();
        assert_eq!(field.char, 4);
        assert_eq!(field.get_selected().unwrap(), "data");
    }
}
