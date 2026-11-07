use core::fmt::{self, Write};

pub struct FixedBuf<const N: usize> {
    pub buf: [u8; N],
    pub len: usize,
}

impl<const N: usize> FixedBuf<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_bytes()).unwrap_or("")
    }

    pub fn push_byte(&mut self, b: u8) {
        if self.len < N {
            self.buf[self.len] = b;
            self.len += 1;
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        let avail = N.saturating_sub(self.len);
        let to_copy = if bytes.len() < avail { bytes.len() } else { avail };
        let mut i = 0;
        while i < to_copy {
            self.buf[self.len + i] = bytes[i];
            i += 1;
        }
        self.len += to_copy;
    }

    pub fn push_str(&mut self, s: &str) {
        self.push_bytes(s.as_bytes());
    }

    pub fn set_bytes(&mut self, bytes: &[u8]) {
        self.clear();
        self.push_bytes(bytes);
    }
}

impl<const N: usize> Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

pub fn trim_ascii(mut s: &[u8]) -> &[u8] {
    while let Some(&first) = s.first() {
        if first == b' ' || first == b'\t' || first == b'\r' || first == b'\n' {
            s = &s[1..];
        } else {
            break;
        }
    }
    while let Some(&last) = s.last() {
        if last == b' ' || last == b'\t' || last == b'\r' || last == b'\n' {
            s = &s[..s.len() - 1];
        } else {
            break;
        }
    }
    s
}

pub fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !a[i].eq_ignore_ascii_case(&b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

pub fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    let limit = haystack.len() - needle.len();
    let mut i = 0;
    while i <= limit {
        if eq_ignore_ascii_case(&haystack[i..i + needle.len()], needle) {
            return true;
        }
        i += 1;
    }
    false
}

pub fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.len() < needle.len() {
        return None;
    }
    let limit = haystack.len() - needle.len();
    let mut i = 0;
    while i <= limit {
        let mut matched = true;
        let mut j = 0;
        while j < needle.len() {
            if haystack[i + j] != needle[j] {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub fn rfind_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }
    if haystack.len() < needle.len() {
        return None;
    }
    let mut i = haystack.len() - needle.len();
    loop {
        let mut matched = true;
        let mut j = 0;
        while j < needle.len() {
            if haystack[i + j] != needle[j] {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return Some(i);
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}

/// Converts a null-terminated or bounded UTF-16 slice into UTF-8 inside `out`.
/// Returns the number of UTF-8 bytes written.
#[cfg(target_os = "windows")]
pub fn utf16_to_utf8(utf16: &[u16], out: &mut [u8]) -> usize {
    let mut len = 0;
    while len < utf16.len() && utf16[len] != 0 {
        len += 1;
    }
    let mut pos = 0;
    for ch in core::char::decode_utf16(utf16[..len].iter().copied()) {
        let c = ch.unwrap_or('\u{FFFD}');
        let mut tmp = [0u8; 4];
        let encoded = c.encode_utf8(&mut tmp).as_bytes();
        if pos + encoded.len() > out.len() {
            break;
        }
        let mut k = 0;
        while k < encoded.len() {
            out[pos + k] = encoded[k];
            k += 1;
        }
        pos += encoded.len();
    }
    pos
}

/// Converts a UTF-8 slice into a null-terminated UTF-16 buffer inside `out`.
/// Returns the number of u16 elements written (excluding the trailing null terminator).
#[cfg(target_os = "windows")]
pub fn utf8_to_utf16_null(utf8: &[u8], out: &mut [u16]) -> usize {
    if out.is_empty() {
        return 0;
    }
    let s = core::str::from_utf8(utf8).unwrap_or("");
    let mut pos = 0;
    for c in s.encode_utf16() {
        if pos + 1 >= out.len() {
            break;
        }
        out[pos] = c;
        pos += 1;
    }
    out[pos] = 0;
    pos
}

fn unescape_json_string(raw: &[u8], out: &mut [u8]) -> usize {
    let mut i = 0;
    let mut pos = 0;
    while i < raw.len() && pos < out.len() {
        if raw[i] == b'"' {
            break;
        }
        if raw[i] == b'\\' && i + 1 < raw.len() {
            i += 1;
            let b = match raw[i] {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'\\' => b'\\',
                b'/' => b'/',
                b'"' => b'"',
                other => other,
            };
            out[pos] = b;
            pos += 1;
        } else {
            out[pos] = raw[i];
            pos += 1;
        }
        i += 1;
    }
    pos
}

/// Finds `"<key>"` in `json` and extracts its string value `"<val>"` into `out` (unescaping `\\` and `\/`).
pub fn json_extract_string_value(json: &[u8], key: &[u8], out: &mut [u8]) -> usize {
    let mut search_pos = 0;
    while search_pos < json.len() {
        let Some(rel) = find_subslice(&json[search_pos..], key) else {
            return 0;
        };
        let key_start = search_pos + rel;
        let key_end = key_start + key.len();
        search_pos = key_end;

        if key_start == 0 || json[key_start - 1] != b'"' || key_end >= json.len() || json[key_end] != b'"' {
            continue;
        }

        let mut i = key_end + 1;
        while i < json.len() && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n') {
            i += 1;
        }
        if i >= json.len() || json[i] != b':' {
            continue;
        }
        i += 1;
        while i < json.len() && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n') {
            i += 1;
        }
        if i >= json.len() || json[i] != b'"' {
            continue;
        }
        i += 1;
        return unescape_json_string(&json[i..], out);
    }
    0
}

/// Finds `"<key>" : [ "<first_item>" ]` in `json` and extracts `<first_item>` into `out`.
pub fn json_extract_first_array_string(json: &[u8], key: &[u8], out: &mut [u8]) -> usize {
    let mut search_pos = 0;
    while search_pos < json.len() {
        let Some(rel) = find_subslice(&json[search_pos..], key) else {
            return 0;
        };
        let key_start = search_pos + rel;
        let key_end = key_start + key.len();
        search_pos = key_end;

        if key_start == 0 || json[key_start - 1] != b'"' || key_end >= json.len() || json[key_end] != b'"' {
            continue;
        }

        let mut i = key_end + 1;
        while i < json.len() && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n') {
            i += 1;
        }
        if i >= json.len() || json[i] != b':' {
            continue;
        }
        i += 1;
        while i < json.len() && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n') {
            i += 1;
        }
        if i >= json.len() || json[i] != b'[' {
            continue;
        }
        i += 1;
        while i < json.len() && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n') {
            i += 1;
        }
        if i >= json.len() || json[i] != b'"' {
            continue;
        }
        i += 1;
        return unescape_json_string(&json[i..], out);
    }
    0
}

