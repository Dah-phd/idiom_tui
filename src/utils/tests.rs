use crate::utils::chunks::ByteChunks;

use super::{CharLimitedWidths, StrChunks, UTFSafe, UTFSafeStringExt, WriteChunks};
const TEXT: &str = "123🚀13";

#[test]
fn test_insert_str_at_char() {
    let mut s = String::new();
    let mut s_utf8 = String::new();
    let mut s_utf16 = String::new();
    s.insert_str_at_char(0, TEXT);
    let utf8_idx = s_utf8.insert_str_at_char_with_utf8_idx(0, TEXT);
    let utf16_idx = s_utf16.insert_str_at_char_with_utf16_idx(0, TEXT);
    assert_eq!(s, TEXT);
    assert_eq!(s_utf8, TEXT);
    assert_eq!(utf8_idx, 0);
    assert_eq!(s_utf16, TEXT);
    assert_eq!(utf16_idx, 0);
    s.insert_str_at_char(4, TEXT);
    let utf8_idx = s_utf8.insert_str_at_char_with_utf8_idx(4, TEXT);
    let utf16_idx = s_utf16.insert_str_at_char_with_utf16_idx(4, TEXT);
    assert!(&s == "123🚀123🚀1313");
    assert_eq!(s, s_utf8);
    assert_eq!(utf8_idx, 7);
    assert_eq!(s, s_utf16);
    assert_eq!(utf16_idx, 5);
}

#[test]
fn test_insert_str_at_char_last() {
    let mut s = String::from("🚀1");
    let mut s_utf8 = String::from("🚀1");
    let mut s_utf16 = String::from("🚀1");
    s.insert_str_at_char(2, "1🚀");
    assert_eq!(s_utf8.insert_str_at_char_with_utf8_idx(2, "1🚀"), 5);
    assert_eq!(s_utf16.insert_str_at_char_with_utf16_idx(2, "1🚀"), 3);
    assert_eq!(&s, "🚀11🚀");
    assert_eq!(&s, &s_utf8);
    assert_eq!(&s, &s_utf16);
}

#[test]
#[should_panic]
fn test_insert_str_at_char_panic() {
    let mut s = String::from("🚀1");
    s.insert_str_at_char(3, "1🚀");
}

#[test]
#[should_panic]
fn test_insert_str_at_char_utf8_panic() {
    let mut s = String::from("🚀1");
    s.insert_str_at_char_with_utf8_idx(3, "1🚀");
}

#[test]
#[should_panic]
fn test_insert_str_at_char_utf16_panic() {
    let mut s = String::from("🚀1");
    s.insert_str_at_char_with_utf16_idx(3, "1🚀");
}

#[test]
fn test_insert_at_char() {
    let mut s = String::new();
    let mut s_utf8 = String::new();
    let mut s_utf16 = String::new();
    s.insert_at_char(0, '🚀');
    s_utf8.insert_at_char_with_utf8_idx(0, '🚀');
    s_utf16.insert_at_char_with_utf16_idx(0, '🚀');
    assert!(&s == "🚀");
    assert!(&s_utf8 == "🚀");
    assert!(&s_utf16 == "🚀");

    s.insert_at_char(1, '🚀');
    s.insert_at_char(2, 'r');
    assert!(&s == "🚀🚀r");

    assert_eq!(s_utf8.insert_at_char_with_utf8_idx(1, '🚀'), 4);
    assert_eq!(s_utf8.insert_at_char_with_utf8_idx(2, 'r'), 8);
    assert!(&s_utf8 == "🚀🚀r");

    assert_eq!(s_utf16.insert_at_char_with_utf16_idx(1, '🚀'), 2);
    assert_eq!(s_utf16.insert_at_char_with_utf16_idx(2, 'r'), 4);
    assert!(&s_utf16 == "🚀🚀r");
}

#[test]
fn test_insert_at_char_last() {
    let mut s = String::from("🚀1");
    let mut s_utf8 = String::from("🚀1");
    let mut s_utf16 = String::from("🚀1");
    s.insert_at_char(2, '🚀');
    assert_eq!(s_utf8.insert_at_char_with_utf8_idx(2, '🚀'), 5);
    assert_eq!(s_utf16.insert_at_char_with_utf16_idx(2, '🚀'), 3);
    assert_eq!(&s, "🚀1🚀");
    assert_eq!(&s, &s_utf8);
    assert_eq!(&s, &s_utf16);
}

#[test]
#[should_panic]
fn test_insert_at_char_panic() {
    let mut s = String::from("🚀1");
    s.insert_at_char(3, '🚀');
}

#[test]
#[should_panic]
fn test_insert_at_char_utf8_panic() {
    let mut s = String::from("🚀1");
    s.insert_at_char_with_utf8_idx(3, '🚀');
}

#[test]
#[should_panic]
fn test_insert_at_char_utf16_panic() {
    let mut s = String::from("🚀1");
    s.insert_at_char_with_utf16_idx(3, '🚀');
}

#[test]
#[should_panic]
fn test_truncate() {
    let mut s = String::from(TEXT);
    s.truncate(4);
}

#[test]
fn test_truncate_utf8() {
    assert_eq!((4, "123"), "123".truncate_width(7));
    assert_eq!((1, "123"), TEXT.truncate_width(4));
    assert_eq!(3, TEXT.truncate_width(4).1.len());
    assert_eq!((0, "123🚀"), TEXT.truncate_width(5));
    assert_eq!(7, TEXT.truncate_width(5).1.len());
    assert_eq!(4, TEXT.truncate_width(5).1.chars().count());
    assert_eq!((0, "🚀13"), TEXT.truncate_width_start(4));
    assert_eq!((1, "13"), TEXT.truncate_width_start(3));
}

#[test]
fn test_width_split() {
    assert_eq!("🚀13".width_split(2), ("🚀", Some("13")));
    assert_eq!("🚀13".width_split(1), ("", Some("🚀13")));
    assert_eq!("🚀13".width_split(4), ("🚀13", None));
    assert_eq!("🚀13".width_split(0), ("", Some("🚀13")));
    assert_eq!("🚀13".width_split(3000), ("🚀13", None));
    assert_eq!("🚀13🚀13".width_split(5), ("🚀13", Some("🚀13")));
    assert_eq!("🚀13🚀13".width_split(6), ("🚀13🚀", Some("13")));
}

#[test]
fn test_width_split_string() {
    assert_eq!(String::from("🚀13").width_split(2), ("🚀", Some("13")));
    assert_eq!(String::from("🚀13").width_split(1), ("", Some("🚀13")));
    assert_eq!(String::from("🚀13").width_split(4), ("🚀13", None));
    assert_eq!(String::from("🚀13").width_split(0), ("", Some("🚀13")));
    assert_eq!(String::from("🚀13").width_split(3000), ("🚀13", None));
    assert_eq!(
        String::from("🚀13🚀13").width_split(5),
        ("🚀13", Some("🚀13"))
    );
    assert_eq!(
        String::from("🚀13🚀13").width_split(6),
        ("🚀13🚀", Some("13"))
    );
}

#[test]
#[should_panic]
fn test_split_std() {
    let _ = TEXT.split_at(4);
}

#[test]
fn test_split_utf8() {
    assert_eq!(TEXT.split_at(3), TEXT.split_at_char(3));
    assert_eq!(("123🚀", "13"), TEXT.split_at_char(4));
}

/// example issue
#[test]
#[should_panic]
fn test_utf8_split_off_panic() {
    let mut s = String::from(TEXT);
    let _ = s.split_off(4);
}

#[test]
#[should_panic]
fn test_utf8_split_off_out_of_bounds() {
    let mut s = String::from(TEXT);
    s.split_off_at_char(30);
}

#[test]
fn test_utf8_split_off() {
    let mut s = String::from(TEXT);
    assert_eq!(s.split_off_at_char(4), String::from("13"));
    assert_eq!(s, "123🚀");
}

/// example issue
#[test]
#[should_panic]
fn test_replace_range() {
    let mut s = String::from(TEXT);
    s.replace_range(4.., ""); // in char boundry
}

#[test]
fn test_utf8_replace_range() {
    let mut s = String::new();
    s.replace_range(0..0, "asd");
    assert!(&s == "asd");
    s.clear();
    s.replace_char_range(0..0, "🚀🚀");
    assert_eq!(&s, "🚀🚀");
    s.replace_char_range(1..2, "asd");
    assert_eq!(&s, "🚀asd");
}

#[test]
#[should_panic]
fn test_utf8_replace_range_panic() {
    let mut s = String::new();
    s.replace_char_range(0..1, "panic");
}

#[test]
fn test_replace_from() {
    let mut s = String::from("text");
    s.replace_from_char(0, "123");
    assert!(&s == "123");
    s.clear();
    s.replace_from_char(0, "123");
    assert!(&s == "123");
}

#[test]
fn test_replace_till() {
    let mut s = String::from("🚀🚀");
    s.replace_till_char(1, "asd");
    assert!(&s == "asd🚀");
    s.clear();
    s.replace_till_char(0, "🚀");
    assert_eq!(&s, "🚀");
}

#[test]
fn test_utf8_replaces() {
    let mut s = String::from(TEXT);
    let mut std_s = s.clone();
    s.replace_from_char(4, "replace_with");
    std_s.replace_range(7.., "replace_with");
    assert_eq!(s, std_s);
}

#[test]
fn test_utf8_str() {
    assert_eq!(TEXT.len(), 9);
    assert_eq!(TEXT.char_len(), 6);
    assert_eq!(TEXT.width(), 7);
}

/// represent issue solved by UTF8 traits
#[test]
#[should_panic]
fn test_std_remove() {
    let mut s = String::from(TEXT);
    s.remove(4); // in char boundry
}

#[test]
fn test_utf8_remove() {
    let mut s = String::from(TEXT);
    let mut s_utf8 = String::from(TEXT);
    let mut s_utf16 = String::from(TEXT);

    assert_eq!(s.len(), 9);
    assert_eq!(s.char_len(), 6);
    assert_eq!(s.width(), 7);
    assert_eq!(s.remove_at_char(4), '1');
    assert_eq!(s_utf8.remove_at_char_with_utf8_idx(4), (7, '1'));
    assert_eq!(s_utf16.remove_at_char_with_utf16_idx(4), (5, '1'));
    assert_eq!(s.remove_at_char(3), '🚀');
    assert_eq!(s_utf8.remove_at_char_with_utf8_idx(3), (3, '🚀'));
    assert_eq!(s_utf16.remove_at_char_with_utf16_idx(3), (3, '🚀'));
    assert_eq!(&s, "1233");
}

#[test]
fn test_char_remove_last() {
    let mut s = String::from("🚀");
    let r = s.remove_at_char(0);
    assert_eq!(r, '🚀')
}

#[test]
#[should_panic]
fn test_char_remove_panics() {
    let mut s = String::from("1");
    s.remove_at_char(1);
}

#[test]
fn test_utf8_remove_last() {
    let mut s = String::from("🚀");
    let r = s.remove_at_char_with_utf8_idx(0);
    assert_eq!(r, (0, '🚀'))
}

#[test]
#[should_panic]
fn test_utf8_remove_panics() {
    let mut s = String::from("1");
    s.remove_at_char_with_utf8_idx(1);
}

#[test]
fn test_utf16_remove_last() {
    let mut s = String::from("🚀");
    let r = s.remove_at_char_with_utf16_idx(0);
    assert_eq!(r, (0, '🚀'))
}

#[test]
#[should_panic]
fn test_utf16_remove_panics() {
    let mut s = String::from("1");
    s.remove_at_char_with_utf16_idx(1);
}

#[test]
fn test_utf8_get() {
    assert_eq!(TEXT.get_char_range(0, 10), None);
    assert_eq!(TEXT.get_char_range(0, 3), Some("123"));
    assert_eq!(TEXT.get_char_range(3, 4), Some("🚀"));
}

#[test]
fn test_utf8_get_from() {
    assert_eq!(TEXT.get_from_char(10), None);
    assert_eq!(TEXT.get_from_char(0), Some(TEXT));
    assert_eq!(TEXT.get_from_char(3), Some("🚀13"));
    assert_eq!(TEXT.get_from_char(4), Some("13"));
}

#[test]
fn test_utf8_get_till() {
    assert_eq!(TEXT.get_to_char(10), None);
    assert_eq!(TEXT.get_to_char(3), Some("123"));
    assert_eq!(TEXT.get_to_char(4), Some("123🚀"));
}

#[test]
#[should_panic]
fn test_utf8_remove_panic() {
    let mut s = String::new();
    s.remove_at_char(0);
}

#[test]
fn test_chunks() {
    let text = "123🚀asdas123123123afsadasras";
    let mut chunks = WriteChunks::new(text, 4);
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 3,
            text: "123"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "🚀as"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "das1"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "2312"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "3123"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "afsa"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "dasr"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 2,
            text: "as"
        })
    );
    assert_eq!(chunks.next(), None);
}

#[test]
fn test_chunks_short() {
    let text = "123";
    let mut chunks = WriteChunks::new(text, 5);
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 3,
            text: "123"
        })
    );
    assert_eq!(chunks.next(), None);
}

#[test]
fn test_chunks_byte() {
    let text = "123asdas123123123afsadasras";
    let mut chunks = ByteChunks::new(text, 4);
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "123a"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "sdas"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "1231"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "2312"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "3afs"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 4,
            text: "adas"
        })
    );
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 3,
            text: "ras"
        })
    );
    assert_eq!(chunks.next(), None);
}

#[test]
fn test_chunks_byte_short() {
    let text = "123";
    let mut chunks = ByteChunks::new(text, 5);
    assert_eq!(
        chunks.next(),
        Some(StrChunks {
            width: 3,
            text: "123"
        })
    );
    assert_eq!(chunks.next(), None);
}

#[test]
fn test_char_limited_chunk() {
    let text = "🚀a";
    let mut chunks = CharLimitedWidths::new(text, 2);
    assert_eq!(chunks.next(), Some(('🚀', 2)));
    assert_eq!(chunks.next(), Some(('a', 1)));
    assert_eq!(chunks.next(), None);
    let mut chunks = CharLimitedWidths::new(text, 1);
    assert_eq!(chunks.next(), Some(('‽', 1)));
    assert_eq!(chunks.next(), Some(('a', 1)));
    assert_eq!(chunks.next(), None);
}
