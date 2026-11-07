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

/// Pure-#![no_std] Windows command-line parser (replaces Shell32's `CommandLineToArgvW`).
/// Skips `argv[0]` (the executable path) and writes up to `out_args.len()` UTF-8 arguments.
#[cfg(target_os = "windows")]
pub fn parse_windows_cmdline_utf16(cmd: &[u16], out_args: &mut [FixedBuf<128>; 8]) -> usize {
    let mut i = 0usize;
    let n = cmd.len();

    // 1. Skip leading whitespace
    while i < n && (cmd[i] == b' ' as u16 || cmd[i] == b'\t' as u16) {
        i += 1;
    }
    if i >= n {
        return 0;
    }

    // 2. Skip argv[0] (executable name, may be quoted)
    if cmd[i] == b'"' as u16 {
        i += 1;
        while i < n && cmd[i] != b'"' as u16 {
            i += 1;
        }
        if i < n {
            i += 1;
        }
    } else {
        while i < n && cmd[i] != b' ' as u16 && cmd[i] != b'\t' as u16 {
            i += 1;
        }
    }

    // 3. Parse subsequent arguments following standard Windows C/C++ argv rules
    let mut argc = 0usize;
    let mut wtoken = [0u16; 260];

    while i < n && argc < out_args.len() {
        while i < n && (cmd[i] == b' ' as u16 || cmd[i] == b'\t' as u16) {
            i += 1;
        }
        if i >= n {
            break;
        }

        let mut wlen = 0usize;
        let mut in_quotes = false;

        while i < n {
            let c = cmd[i];
            if !in_quotes && (c == b' ' as u16 || c == b'\t' as u16) {
                break;
            }
            if c == b'\\' as u16 {
                let mut bs_count = 0usize;
                while i < n && cmd[i] == b'\\' as u16 {
                    bs_count += 1;
                    i += 1;
                }
                if i < n && cmd[i] == b'"' as u16 {
                    // 2N backslashes + quote -> N backslashes, toggle quote
                    // 2N+1 backslashes + quote -> N backslashes + literal quote
                    let pairs = bs_count / 2;
                    let mut p = 0;
                    while p < pairs && wlen < wtoken.len() {
                        wtoken[wlen] = b'\\' as u16;
                        wlen += 1;
                        p += 1;
                    }
                    if bs_count % 2 == 0 {
                        in_quotes = !in_quotes;
                    } else if wlen < wtoken.len() {
                        wtoken[wlen] = b'"' as u16;
                        wlen += 1;
                    }
                    i += 1;
                } else {
                    let mut p = 0;
                    while p < bs_count && wlen < wtoken.len() {
                        wtoken[wlen] = b'\\' as u16;
                        wlen += 1;
                        p += 1;
                    }
                }
            } else if c == b'"' as u16 {
                in_quotes = !in_quotes;
                i += 1;
            } else {
                if wlen < wtoken.len() {
                    wtoken[wlen] = c;
                    wlen += 1;
                }
                i += 1;
            }
        }

        out_args[argc].len = utf16_to_utf8(&wtoken[..wlen], &mut out_args[argc].buf);
        argc += 1;
    }

    argc
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

/// Locates the `"manifest_paths"` array `[` and matching `]` in `json` structurally
/// (respecting JSON string literals and escapes).
fn find_manifest_paths_array_bounds(json: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    while i < json.len() {
        if json[i] == b'"' {
            let str_start = i + 1;
            i += 1;
            while i < json.len() {
                if json[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if json[i] == b'"' {
                    break;
                }
                i += 1;
            }
            let str_end = i;
            i += 1;

            if &json[str_start..str_end] == b"manifest_paths" {
                while i < json.len()
                    && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\r' || json[i] == b'\n')
                {
                    i += 1;
                }
                if i < json.len() && json[i] == b':' {
                    i += 1;
                    while i < json.len()
                        && (json[i] == b' '
                            || json[i] == b'\t'
                            || json[i] == b'\r'
                            || json[i] == b'\n')
                    {
                        i += 1;
                    }
                    if i < json.len() && json[i] == b'[' {
                        let open_bracket = i;
                        i += 1;
                        let mut depth = 1usize;
                        while i < json.len() && depth > 0 {
                            if json[i] == b'"' {
                                i += 1;
                                while i < json.len() {
                                    if json[i] == b'\\' {
                                        i += 2;
                                        continue;
                                    }
                                    if json[i] == b'"' {
                                        break;
                                    }
                                    i += 1;
                                }
                            } else if json[i] == b'[' {
                                depth += 1;
                            } else if json[i] == b']' {
                                depth -= 1;
                                if depth == 0 {
                                    return Some((open_bracket, i));
                                }
                            }
                            i += 1;
                        }
                        return None;
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

fn push_slice_checked(out: &mut [u8], pos: &mut usize, bytes: &[u8]) -> bool {
    if *pos + bytes.len() > out.len() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        out[*pos + i] = bytes[i];
        i += 1;
    }
    *pos += bytes.len();
    true
}

fn push_json_escaped_checked(out: &mut [u8], pos: &mut usize, raw: &[u8]) -> bool {
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\\' {
            if !push_slice_checked(out, pos, b"\\\\") {
                return false;
            }
        } else if raw[i] == b'"' {
            if !push_slice_checked(out, pos, b"\\\"") {
                return false;
            }
        } else if !push_slice_checked(out, pos, &[raw[i]]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Structurally updates `"manifest_paths"` in `appconfig.json` while preserving all other
/// JSON fields before `[` and after `]`.
/// - Filters out any existing entry matching `manifest_filename`.
/// - If `add_manifest_path` is `Some(path)`, appends it as the final element of `"manifest_paths"`.
/// - Returns `Some(written_len)` on success, or `None` if `out` capacity would be exceeded or JSON is malformed.
pub fn update_appconfig_manifest_paths(
    existing: &[u8],
    manifest_filename: &[u8],
    add_manifest_path: Option<&[u8]>,
    out: &mut [u8],
) -> Option<usize> {
    let mut pos = 0usize;
    let trimmed = trim_ascii(existing);

    if trimmed.is_empty() {
        if !push_slice_checked(out, &mut pos, b"{\n   \"manifest_paths\" : [") {
            return None;
        }
        if let Some(p) = add_manifest_path {
            if !push_slice_checked(out, &mut pos, b"\n      \"") {
                return None;
            }
            if !push_json_escaped_checked(out, &mut pos, p) {
                return None;
            }
            if !push_slice_checked(out, &mut pos, b"\"\n   ") {
                return None;
            }
        }
        if !push_slice_checked(out, &mut pos, b"]\n}\n") {
            return None;
        }
        return Some(pos);
    }

    let (open_b, close_b) = find_manifest_paths_array_bounds(existing)?;

    // Copy everything up to and including `[`
    if !push_slice_checked(out, &mut pos, &existing[..=open_b]) {
        return None;
    }

    // Parse all quoted string literals inside `[...]`
    let inner = &existing[open_b + 1..close_b];
    let mut count = 0usize;
    let mut i = 0usize;
    while i < inner.len() {
        if inner[i] == b'"' {
            let lit_start = i;
            i += 1;
            while i < inner.len() {
                if inner[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if inner[i] == b'"' {
                    break;
                }
                i += 1;
            }
            if i >= inner.len() {
                return None; // Unterminated string literal
            }
            let lit_end = i + 1; // inclusive of closing quote
            let raw_literal = &inner[lit_start..lit_end];
            i = lit_end;

            // Keep entry unless it matches our manifest filename
            if !contains_ignore_ascii_case(raw_literal, manifest_filename) {
                if count > 0 {
                    if !push_slice_checked(out, &mut pos, b",") {
                        return None;
                    }
                }
                if !push_slice_checked(out, &mut pos, b"\n      ") {
                    return None;
                }
                if !push_slice_checked(out, &mut pos, raw_literal) {
                    return None;
                }
                count += 1;
            }
        } else {
            i += 1;
        }
    }

    if let Some(p) = add_manifest_path {
        if count > 0 {
            if !push_slice_checked(out, &mut pos, b",") {
                return None;
            }
        }
        if !push_slice_checked(out, &mut pos, b"\n      \"") {
            return None;
        }
        if !push_json_escaped_checked(out, &mut pos, p) {
            return None;
        }
        if !push_slice_checked(out, &mut pos, b"\"") {
            return None;
        }
        count += 1;
    }

    if count > 0 {
        if !push_slice_checked(out, &mut pos, b"\n   ") {
            return None;
        }
    }

    // Copy `]` and everything after it verbatim (preserving all other keys in appconfig.json)
    if !push_slice_checked(out, &mut pos, &existing[close_b..]) {
        return None;
    }

    Some(pos)
}
