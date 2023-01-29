use std::borrow::Cow;

pub(crate) fn skip_whitespace(s: &str) -> (&str, usize) {
    let mut chars = s.chars();
    loop {
        let i = s.len() - chars.as_str().len();
        let Some(c) = chars.next() else {
            return ("", s.len());
        };

        if c != ' ' && c != '\t' {
            break (&s[i..], i);
        }
    }
}

pub(crate) fn till_whitespace(s: &str) -> (&str, usize) {
    let mut chars = s.chars();
    loop {
        let i = s.len() - chars.as_str().len();
        let Some(c) = chars.next() else {
            return (s, s.len());
        };

        if c == ' ' || c == '\t' {
            break (&s[..i], i + 1);
        }
    }
}

pub(crate) fn escape_end_in_str(
    s: &str,
    end_char: char,
    escaped_replacement: Option<u8>,
) -> (Cow<str>, Option<usize>) {
    debug_assert!(end_char.is_ascii());

    let mut escaped_ends = Vec::with_capacity(0);

    if s.is_empty() {
        return (Cow::Owned(String::with_capacity(0)), None);
    }

    let mut escaped = false;

    // 1. Find all the positions of escaped characters
    let mut chars = s.chars();
    let end = loop {
        // Fetch the byte index of the current char
        let i = s.len() - chars.as_str().len();
        let Some(c) = chars.next() else {
            break None;
        };

        if escaped && (c == '\\' || c == end_char) {
            escaped_ends.push(i);
        } else if c == end_char {
            break Some(i);
        }

        escaped = !escaped && c == '\\';
    };

    let (s, end) = if let Some(end) = end {
        (&s[..end], Some(end + 1))
    } else {
        (s, end)
    };

    // If no escaped characters were found, return.
    if escaped_ends.is_empty() {
        return (Cow::Borrowed(s), end);
    }

    // 2. Remove all escaped new lines and replace them with spaces
    let mut s = String::from(s).into_bytes();
    for (index, offset) in escaped_ends.into_iter().enumerate() {
        debug_assert!(offset > index);
        debug_assert!(s.len() >= offset - index);

        debug_assert_eq!(s[offset - index - 1], b'\\');

        if let Some(escaped_replacement) = escaped_replacement {
            s[offset - index] = escaped_replacement;
        }
        s.remove(offset - index - 1);
    }
    let s = unsafe { String::from_utf8_unchecked(s) };

    (Cow::Owned(s), end)
}

/// Takes a string until the end of the line. If an escaped line-feed is encountered it is
/// converted into a space character.
pub(crate) fn till_end_of_line(s: &str) -> (Cow<str>, usize) {
    let (taken, end_of_line) = escape_end_in_str(s, '\n', Some(b' '));
    let end_of_line = end_of_line.unwrap_or(s.len());
    (taken, end_of_line)
}

pub(crate) fn take_control_string(s: &str) -> Option<(&str, usize)> {
    if s.is_empty() {
        return None;
    }

    if !s.starts_with('[') {
        return Some(till_whitespace(s));
    }

    let end_position = s.find(']')?;
    let taken = &s[..end_position + 1];

    Some((taken, end_position + 1))
}

pub(crate) fn take_string(s: &str) -> Option<(Cow<str>, usize)> {
    if s.is_empty() {
        return Some((Cow::Owned(String::with_capacity(0)), 0));
    }

    if !s.starts_with('[') {
        let (taken, start_after) = till_whitespace(s);
        let taken = Cow::Borrowed(taken);
        return Some((taken, start_after));
    }

    let s = &s[1..];
    let (taken, end_of_line) = escape_end_in_str(s, ']', None);
    Some((taken, end_of_line? + 1))
}

pub(crate) fn take_all_strings(s: &str) -> Option<Vec<Cow<str>>> {
    let mut strings = Vec::new();
    let mut start_item = 0;
    loop {
        let item_s = &s[start_item..];
        let (item_s, skipped) = skip_whitespace(item_s);
        if item_s.bytes().next().map_or(true, |c| c == b'\n') {
            return Some(strings);
        }

        start_item += skipped;

        let (taken, length) = take_string(item_s)?;
        if taken.is_empty() {
            return Some(strings);
        }

        strings.push(taken);
        start_item += length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn till_eol() {
        macro_rules! assert_test {
            ($s:literal => $taken:literal, $length:literal) => {
                let (taken, length) = till_end_of_line($s);

                let taken = String::from(taken);

                assert_eq!(taken, $taken);
                assert_eq!(length, $length);
            };
        }

        assert_test!("" => "", 0);
        assert_test!("abc" => "abc", 3);
        assert_test!("abc\n" => "abc", 4);
        assert_test!("abc\\\n" => "abc ", 5);
        assert_test!("abc\\\n\\\n" => "abc  ", 7);
        assert_test!("abc\nxyz" => "abc", 4);
        assert_test!("abc\nxyz\n123" => "abc", 4);
        assert_test!("abc\\\nxyz\n123" => "abc xyz", 9);
    }

    #[test]
    fn skip_ws() {
        macro_rules! assert_test {
            ($s:literal => $start:expr, $taken:literal) => {
                let (taken, start) = skip_whitespace($s);
                assert_eq!(start, $start);
                assert_eq!(taken, $taken);
            };
        }

        assert_test!("" => 0, "");
        assert_test!("  " => 2, "");
        assert_test!("  abc" => 2, "abc");
        assert_test!("  \t  abc" => 5, "abc");
        assert_test!("  \tx abc" => 3, "x abc");
    }

    #[test]
    fn till_ws() {
        macro_rules! assert_test {
            ($s:literal => $taken:literal, $start:literal) => {
                let (taken, start) = till_whitespace($s);

                let taken = String::from(taken);

                assert_eq!(taken, $taken);
                assert_eq!(start, $start);
            };
        }

        assert_test!("" => "", 0);
        assert_test!("abc" => "abc", 3);
        assert_test!("abc xyz" => "abc", 4);
        assert_test!("abc xyz 123" => "abc", 4);
    }

    #[test]
    fn esc_end_in_str() {
        macro_rules! assert_test {
            ($s:literal => !) => {
                assert!(take_string($s).is_none());
            };
            ($s:literal, $needle:literal, $replacement:expr => $taken:literal, $length:expr) => {
                let (taken, length) = escape_end_in_str($s, $needle, $replacement);

                let taken = String::from(taken);

                assert_eq!(taken, $taken);
                assert_eq!(length, $length);
            };
        }

        assert_test!("", '\n', Some(b' ') => "", None);
        assert_test!("abc", '\n', Some(b' ') => "abc", None);
        assert_test!("abc xyz", '\n', Some(b' ') => "abc xyz", None);
        assert_test!("abc xyz\n012", '\n', Some(b' ') => "abc xyz", Some(8));
        assert_test!("abc xyz\\\n012", '\n', Some(b' ') => "abc xyz 012", None);
        assert_test!("[...[...\\]...] 123", ']', None => "[...[...]...", Some(14));
        assert_test!("[ 123", ']', None => "[ 123", None);
    }

    #[test]
    fn take_str() {
        macro_rules! assert_test {
            ($s:literal => !) => {
                assert!(take_string($s).is_none());
            };
            ($s:literal => $taken:literal, $length:literal) => {
                let string = take_string($s);
                assert!(string.is_some());
                let (taken, length) = string.unwrap();

                let taken = String::from(taken);

                assert_eq!(taken, $taken);
                assert_eq!(length, $length);
            };
        }

        assert_test!("" => "", 0);
        assert_test!("abc" => "abc", 3);
        assert_test!("abc xyz" => "abc", 4);
        assert_test!("[abc xyz] 123" => "abc xyz", 9);
        assert_test!("[abc \\]xyz] 123" => "abc ]xyz", 11);
        assert_test!("[...[...\\]...] 123" => "...[...]...", 14);
        assert_test!("[ 123" => !);
    }

    #[test]
    fn take_all_strs() {
        macro_rules! assert_test {
            ($s:literal => !) => {
                assert!(take_all_strings($s).is_none());
            };
            ($s:literal => [ $($taken:literal),* $(,)? ]) => {
                let strings = take_all_strings($s);
                assert!(strings.is_some());
                let strings = strings.unwrap();

                let strings: Vec<String> = strings.into_iter().map(|s| String::from(s)).collect();
                let eq: Vec<&str> = vec![$($taken,)*];

                assert_eq!(strings, eq);
            };
        }

        assert_test!("" => []);
        assert_test!("abc" => ["abc"]);
        assert_test!("abc xyz" => ["abc", "xyz"]);
        assert_test!("abc xyz 123" => ["abc", "xyz", "123"]);
        assert_test!("[abc xyz] 123" => ["abc xyz", "123"]);
        assert_test!("[abc \\]xyz] 123" => ["abc ]xyz", "123"]);
        assert_test!("[...[...\\]...] 123" => ["...[...]...", "123"]);
        assert_test!("[ 123" => !);
    }
}
