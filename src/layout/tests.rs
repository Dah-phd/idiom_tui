use super::{Line, Rect};
use crate::{
    backend::{Backend, StyleExt},
    backend::{MockedBackend, MockedStyle},
    layout::Borders,
};

#[test]
fn split_horizont_rel() {
    let rect: Rect = (20, 30).into();
    assert_eq!(
        rect.split_horizont_rel(12),
        (
            Rect {
                row: 0,
                col: 0,
                width: 12,
                height: 30,
                borders: Borders::empty()
            },
            Rect {
                row: 0,
                col: 12,
                width: 8,
                height: 30,
                borders: Borders::empty()
            }
        )
    );
}

#[test]
fn split_horizont_rel_overflow() {
    let rect: Rect = (10, 30).into();
    assert_eq!(
        rect.split_horizont_rel(12),
        (
            Rect {
                row: 0,
                col: 0,
                width: 10,
                height: 30,
                borders: Borders::empty()
            },
            Rect {
                row: 0,
                col: 10,
                width: 0,
                height: 30,
                borders: Borders::empty()
            }
        )
    );
}

#[test]
fn split_vertical_rel() {
    let rect = Rect::from((20, 30));
    assert_eq!(
        rect.split_vertical_rel(12),
        (
            Rect {
                row: 0,
                col: 0,
                width: 20,
                height: 12,
                borders: Borders::empty()
            },
            Rect {
                row: 12,
                col: 0,
                width: 20,
                height: 18,
                borders: Borders::empty()
            }
        )
    );
}

#[test]
fn split_vertical_rel_overflow() {
    let rect = Rect::from((20, 10));
    assert_eq!(
        rect.split_vertical_rel(12),
        (
            Rect {
                row: 0,
                col: 0,
                width: 20,
                height: 10,
                borders: Borders::empty()
            },
            Rect {
                row: 10,
                col: 0,
                width: 20,
                height: 0,
                borders: Borders::empty()
            }
        )
    );
}

#[test]
fn rect_pop_line() {
    let mut rect: Rect = (30, 10).into();
    let last = rect.clone().into_iter().last();
    let poped = rect.pop_line();
    assert_eq!(
        poped,
        Line {
            row: 9,
            col: 0,
            width: 30
        }
    );
    assert_eq!(Some(poped), last);
    assert_eq!(Some(rect.clone().pop_line()), rect.next_line_back());
}

#[test]
fn rect_next_line_back() {
    let mut rect: Rect = (30, 10).into();
    let last = rect.clone().into_iter().last();
    let next_back = rect.next_line_back();
    assert_eq!(
        next_back,
        Some(Line {
            row: 9,
            col: 0,
            width: 30
        })
    );
    assert_eq!(next_back, last);
    assert_eq!(Some(rect.clone().pop_line()), rect.next_line_back());
}

#[test]
fn render_centered() {
    let width = 50;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered("idiom", &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::default(), "<<padding: 23>>".to_owned()),
            (MockedStyle::default(), "idiom".to_owned()),
            (MockedStyle::default(), "<<padding: 22>>".to_owned())
        ]
    )
}

#[test]
fn render_centered_maxed() {
    let width = 4;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered("idiom", &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::default(), "idio".to_owned()),
        ]
    )
}

#[test]
fn render_centered_one_pad() {
    let width = 6;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered("idiom", &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::default(), "idiom".to_owned()),
            (MockedStyle::default(), "<<padding: 1>>".to_owned())
        ]
    )
}

#[test]
fn render_centered_styled() {
    let width = 7;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered_styled("idiom", MockedStyle::bold(), &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::bold(), "<<set style>>".to_owned()),
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::bold(), "<<padding: 1>>".to_owned()),
            (MockedStyle::bold(), "idiom".to_owned()),
            (MockedStyle::bold(), "<<padding: 1>>".to_owned()),
            (MockedStyle::default(), "<<set style>>".to_owned())
        ]
    );
}

#[test]
fn render_centered_styled_maxed() {
    let width = 4;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered_styled("idiom", MockedStyle::bold(), &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::bold(), "<<set style>>".to_owned()),
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::bold(), "idio".to_owned()),
            (MockedStyle::default(), "<<set style>>".to_owned())
        ]
    );
}

#[test]
fn render_centered_styled_one_pad() {
    let width = 6;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered_styled("idiom", MockedStyle::bold(), &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::bold(), "<<set style>>".to_owned()),
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::bold(), "idiom".to_owned()),
            (MockedStyle::bold(), "<<padding: 1>>".to_owned()),
            (MockedStyle::default(), "<<set style>>".to_owned())
        ]
    );
}

#[test]
fn render_centered_complex() {
    let width = 50;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered("🔥idiom🔥", &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::default(), "<<padding: 21>>".to_owned()),
            (MockedStyle::default(), "🔥idiom🔥".to_owned()), // 5 + 2 + 2 = 9  >>> 50 - 9 = 21 + 20
            (MockedStyle::default(), "<<padding: 20>>".to_owned()),
        ]
    )
}

#[test]
fn render_centered_complex_maxed() {
    let width = 8;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered("🔥idiom🔥", &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::default(), "🔥idiom".to_owned()), // 5 + 2 >> 8 - 7 = 1 pad
            (MockedStyle::default(), "<<padding: 1>>".to_owned()),
        ]
    )
}

#[test]
fn render_centered_complex_style_maxed() {
    let width = 8;
    let line = Line {
        row: 1,
        col: 3,
        width,
    };
    let mut backend = MockedBackend::init();
    line.render_centered_styled("🔥idiom🔥", MockedStyle::bold(), &mut backend);
    assert_eq!(
        backend.drain(),
        [
            (MockedStyle::bold(), "<<set style>>".to_owned()),
            (MockedStyle::default(), "<<go to row: 1 col: 3>>".to_owned()),
            (MockedStyle::bold(), "🔥idiom".to_owned()), // 5 + 2 >> 8 - 7 = 1 pad
            (MockedStyle::bold(), "<<padding: 1>>".to_owned()),
            (MockedStyle::default(), "<<set style>>".to_owned()),
        ]
    )
}

#[test]
fn test_rel_modal() {
    let rect = Rect::new(0, 0, 80, 30);
    assert_ne!(
        rect.clone().pop_line().row,
        rect.modal_relative(26, 10, 20, 7).pop_line().row
    );
    assert_eq!(
        rect.clone().pop_line().row,
        rect.modal_relative(25, 10, 20, 7).pop_line().row
    );
    assert_eq!(
        rect.clone().pop_line().row,
        rect.modal_relative(24, 10, 20, 7).pop_line().row
    );
    assert_eq!(
        rect.clone().pop_line().row,
        rect.modal_relative(23, 10, 20, 7).pop_line().row
    );
}
