pub mod text_field;

pub mod backend;
pub mod layout;
pub mod utils;
pub mod widgets;

pub use backend::Backend;
pub use utils::{ByteChunks, CharLimitedWidths, StrChunks, UTFSafe, UTFSafeStringExt, WriteChunks};

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
