use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use crossterm::style::{Color, ContentStyle};
use idiom_tui::backend::{Backend, CrossTerm, StyleExt};
use idiom_tui::text_field::{Status, TextField};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let cursor_style = ContentStyle::reversed();
    let select_style = ContentStyle::bg(Color::Rgb {
        r: 72,
        g: 72,
        b: 72,
    });

    let mut backend = CrossTerm::init();
    let screen = CrossTerm::screen()?;
    let mut text_field = TextField::default();

    let line = screen.get_line(1).unwrap();
    text_field.widget(line, cursor_style, select_style, &mut backend);

    loop {
        if poll(Duration::from_millis(1_000))? {
            match read()? {
                Event::Key(key) => {
                    let Some(result) = text_field.map(key) else {
                        if matches!(
                            key,
                            KeyEvent {
                                code: KeyCode::Esc,
                                ..
                            }
                        ) {
                            return Ok(());
                        }
                        let line = screen.get_line(2).unwrap();
                        line.render("Not mapped", &mut backend);
                        continue;
                    };
                    match result {
                        Status::Skipped => {
                            let line = screen.get_line(2).unwrap();
                            line.render("skipped", &mut backend);
                        }
                        Status::Updated => {
                            let line = screen.get_line(1).unwrap();
                            text_field.widget(line, cursor_style, select_style, &mut backend);
                            let line = screen.get_line(2).unwrap();
                            line.render("Upd text", &mut backend);
                        }
                        Status::UpdatedCursor => {
                            let line = screen.get_line(1).unwrap();
                            text_field.widget(line, cursor_style, select_style, &mut backend);
                            let line = screen.get_line(2).unwrap();
                            line.render("Upd cursor", &mut backend);
                        }
                    }
                }
                Event::Resize(..) => break,
                _ => (),
            };
            backend.flush_buf();
        }
    }

    Ok(())
}
