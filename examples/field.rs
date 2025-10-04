use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
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
        backend.flush_buf();
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(key) => {
                    let Some(result) = text_field.map(key) else {
                        let msg = match key {
                            KeyEvent {
                                code: KeyCode::Char('C' | 'c'),
                                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                                ..
                            } => {
                                if let Some(text) = text_field.copy() {
                                    format!("Copied {text}")
                                } else {
                                    "Failed copy".to_owned()
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('X' | 'x'),
                                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                                ..
                            } => {
                                if let Some(text) = text_field.cut() {
                                    let line = screen.get_line(1).unwrap();
                                    text_field.widget(
                                        line,
                                        cursor_style,
                                        select_style,
                                        &mut backend,
                                    );
                                    format!("Cut {text}")
                                } else {
                                    "Failed cut".to_owned()
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('e' | 'E'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                text_field.select_token_at_cursor();
                                let line = screen.get_line(1).unwrap();
                                text_field.widget(line, cursor_style, select_style, &mut backend);
                                "Select token range".to_owned()
                            }
                            KeyEvent {
                                code: KeyCode::Esc, ..
                            } => return Ok(()),
                            _ => "Not mapped".to_owned(),
                        };
                        let line = screen.get_line(2).unwrap();
                        line.render(&msg, &mut backend);
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
                Event::Paste(clip) => {
                    if text_field.paste_passthrough(clip).is_updated() {
                        let line = screen.get_line(1).unwrap();
                        text_field.widget(line, cursor_style, select_style, &mut backend);
                        let line = screen.get_line(2).unwrap();
                        line.render("Paste", &mut backend);
                    } else {
                        let line = screen.get_line(2).unwrap();
                        line.render("Failed paste", &mut backend);
                    }
                }
                Event::Resize(..) => break,
                _ => (),
            };
        }
    }

    Ok(())
}
