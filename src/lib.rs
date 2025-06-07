pub mod backend;
pub mod layout;
pub mod state;
#[cfg(feature = "crossterm_backend")]
pub mod text_field;
pub mod utils;
pub mod widgets;

pub use utils::UTF8Safe;

/// This can easily gorow to be a framework itself
pub fn count_as_string(len: usize) -> String {
    if len < 10 {
        format!("  {len}")
    } else if len < 100 {
        format!(" {len}")
    } else {
        String::from("99+")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}
