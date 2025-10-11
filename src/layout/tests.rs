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
    let last = rect.into_iter().last();
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
    let last = rect.into_iter().last();
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
fn relative_modal() {
    let base = Rect::new(1, 43, 241, 67);
    let width = 70;

    let col_offset = 150;
    let rel = base.modal_relative(32, col_offset, width, 7);
    assert_eq!(rel.col, base.col + col_offset);
    assert_eq!(rel.width, width);

    let col_offset = 202;
    let rel = base.modal_relative(32, col_offset, width, 7);
    assert_eq!(rel.col, base.col + col_offset);
    assert_eq!(rel.width, base.width - col_offset as usize);

    let col_offset = 210;
    let rel = base.modal_relative(32, col_offset, width, 7);
    assert_eq!(rel.col, base.col + col_offset);
    assert_eq!(rel.width, base.width - col_offset as usize);

    let rel = base.modal_relative(32, 215, width, 7);
    assert_eq!(rel.col, 254);
    assert_eq!(rel.width, 30);
}

#[test]
fn test_rel_modal() {
    let rect = Rect::new(0, 0, 80, 30);
    assert_eq!(25, rect.modal_relative(26, 10, 20, 7).pop_line().row);

    let mut modal = rect.modal_relative(25, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 26);
    assert_eq!(modal.height, 3);

    let mut modal = rect.modal_relative(24, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 25);
    assert_eq!(modal.height, 4);

    let mut modal = rect.modal_relative(23, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 24);
    assert_eq!(modal.height, 5);

    // reversed

    let mut modal = rect.modal_relative(26, 10, 20, 7);
    assert_eq!(25, modal.pop_line().row);
    assert_eq!(modal.row, 19);
    assert_eq!(modal.height, 6);

    let mut modal = rect.modal_relative(29, 10, 20, 7);
    assert_eq!(28, modal.pop_line().row);
    assert_eq!(modal.row, 22);
    assert_eq!(modal.height, 6);

    // outside boundries

    let mut modal = rect.modal_relative(30, 10, 20, 7);
    assert_eq!(31, modal.pop_line().row);
    assert_eq!(modal.row, 31);
    assert_eq!(modal.height, 0);
}

#[test]
fn test_rel_modal2() {
    let rect = Rect::new(10, 0, 80, 30);
    assert_eq!(35, rect.modal_relative(26, 10, 20, 7).pop_line().row);

    let mut modal = rect.modal_relative(25, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 36);
    assert_eq!(modal.height, 3);

    let mut modal = rect.modal_relative(24, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 35);
    assert_eq!(modal.height, 4);

    let mut modal = rect.modal_relative(23, 10, 20, 7);
    assert_eq!(rect.clone().pop_line().row, modal.pop_line().row);
    assert_eq!(modal.row, 34);
    assert_eq!(modal.height, 5);

    // reversed

    let mut modal = rect.modal_relative(26, 10, 20, 7);
    assert_eq!(35, modal.pop_line().row);
    assert_eq!(modal.row, 29);
    assert_eq!(modal.height, 6);

    let mut modal = rect.modal_relative(29, 10, 20, 7);
    assert_eq!(38, modal.pop_line().row);
    assert_eq!(modal.row, 32);
    assert_eq!(modal.height, 6);

    // outside boundries

    let mut modal = rect.modal_relative(30, 10, 20, 7);
    assert_eq!(41, modal.pop_line().row);
    assert_eq!(modal.row, 41);
    assert_eq!(modal.height, 0);
}

#[test]
fn last_rel_modal_row() {
    let rect = Rect::new(10, 0, 80, 5);

    let modal = rect.modal_relative(0, 10, 20, 7);
    assert_eq!(modal, Rect::new(11, 10, 20, 4));

    let modal = rect.modal_relative(1, 10, 20, 7);
    assert_eq!(modal, Rect::new(12, 10, 0, 0));

    let modal = rect.modal_relative(2, 10, 20, 7);
    assert_eq!(modal, Rect::new(13, 10, 0, 0));

    let modal = rect.modal_relative(3, 10, 20, 7);
    assert_eq!(modal, Rect::new(10, 10, 20, 3));

    let modal = rect.modal_relative(4, 10, 20, 7);
    assert_eq!(modal, Rect::new(10, 10, 20, 4));

    let modal = rect.modal_relative(5, 10, 20, 7);
    assert_eq!(modal, Rect::new(16, 10, 0, 0));
}

#[test]
fn right_top_cornet() {
    let rect = Rect::new(0, 0, 40, 2).right_top_corner(5, 60);
    assert_eq!(Rect::new(0, 0, 40, 2), rect);
    let rect = Rect::new(0, 0, 100, 20).right_top_corner(5, 60);
    assert_eq!(Rect::new(0, 40, 60, 5), rect);
}

#[test]
fn left_top_cornet() {
    let rect = Rect::new(0, 0, 40, 2).left_top_corner(5, 60);
    assert_eq!(Rect::new(0, 0, 40, 2), rect);
    let rect = Rect::new(0, 0, 100, 20).left_top_corner(5, 60);
    assert_eq!(Rect::new(0, 0, 60, 5), rect);
}


#[test]
fn right_bot_cornet() {
    let rect = Rect::new(0, 0, 40, 2).right_bot_corner(5, 60);
    assert_eq!(Rect::new(0, 0, 40, 2), rect);
    let rect = Rect::new(0, 0, 100, 20).right_bot_corner(5, 60);
    assert_eq!(Rect::new(15, 40, 60, 5), rect);
}

#[test]
fn left_bot_cornet() {
    let rect = Rect::new(0, 0, 40, 2).left_bot_corner(5, 60);
    assert_eq!(Rect::new(0, 0, 40, 2), rect);
    let rect = Rect::new(0, 0, 100, 20).left_bot_corner(5, 60);
    assert_eq!(Rect::new(15, 0, 60, 5), rect);
}
