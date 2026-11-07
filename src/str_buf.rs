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

/// Locates the top-level `"manifest_paths"` array `[` and matching `]` in `json` structurally
/// (validating top-level `{...}` object structure, string literals, escapes, and depth).
fn find_manifest_paths_array_bounds(json: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    let mut obj_depth = 0usize;
    let mut arr_depth = 0usize;
    let mut saw_root_open = false;
    let mut saw_non_ws_outside_root = false;
    let mut found_bounds: Option<(usize, usize)> = None;

    while i < json.len() {
        let b = json[i];
        if !saw_root_open
            && b != b'{'
            && b != b' '
            && b != b'\t'
            && b != b'\r'
            && b != b'\n'
        {
            saw_non_ws_outside_root = true;
            break;
        }
        if b == b'"' {
            let str_start = i + 1;
            i += 1;
            while i < json.len() {
                if json[i] == b'\\' {
                    if i + 1 >= json.len() {
                        return None; // Dangling escape at EOF
                    }
                    i += 2;
                    continue;
                }
                if json[i] == b'"' {
                    break;
                }
                i += 1;
            }
            if i >= json.len() {
                return None; // Unterminated string literal
            }
            let str_end = i;
            i += 1;

            // Only match `"manifest_paths"` when it is a top-level key of the root JSON object
            if obj_depth == 1 && arr_depth == 0 && &json[str_start..str_end] == b"manifest_paths" {
                let mut j = i;
                while j < json.len()
                    && (json[j] == b' ' || json[j] == b'\t' || json[j] == b'\r' || json[j] == b'\n')
                {
                    j += 1;
                }
                if j < json.len() && json[j] == b':' {
                    i = j + 1;
                    while i < json.len()
                        && (json[i] == b' '
                            || json[i] == b'\t'
                            || json[i] == b'\r'
                            || json[i] == b'\n')
                    {
                        i += 1;
                    }
                    if i >= json.len() || json[i] != b'[' {
                        return None; // `"manifest_paths"` value is not an array
                    }
                    let open_bracket = i;
                    i += 1;
                    let mut depth = 1usize;
                    while i < json.len() && depth > 0 {
                        if json[i] == b'"' {
                            i += 1;
                            while i < json.len() {
                                if json[i] == b'\\' {
                                    if i + 1 >= json.len() {
                                        return None;
                                    }
                                    i += 2;
                                    continue;
                                }
                                if json[i] == b'"' {
                                    break;
                                }
                                i += 1;
                            }
                            if i >= json.len() {
                                return None; // Unterminated string inside array
                            }
                        } else if json[i] == b'[' {
                            depth += 1;
                        } else if json[i] == b']' {
                            depth -= 1;
                            if depth == 0 {
                                found_bounds = Some((open_bracket, i));
                                i += 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                    if depth != 0 {
                        return None; // Unclosed `[`
                    }
                }
            }
        } else if b == b'{' {
            saw_root_open = true;
            obj_depth += 1;
            i += 1;
        } else if b == b'}' {
            if obj_depth == 0 {
                return None; // Unbalanced `}`
            }
            obj_depth -= 1;
            i += 1;
        } else if b == b'[' {
            arr_depth += 1;
            i += 1;
        } else if b == b']' {
            if arr_depth == 0 {
                return None; // Unbalanced `]`
            }
            arr_depth -= 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    if !saw_root_open || saw_non_ws_outside_root || obj_depth != 0 || arr_depth != 0 {
        return None;
    }
    found_bounds
}

/// Locates the final closing `}` of a valid top-level JSON object when `"manifest_paths"` is absent,
/// plus whether the object already contains at least one key-value pair.
fn find_root_object_insertion_point(json: &[u8]) -> Option<(usize, bool)> {
    let mut i = 0;
    let mut obj_depth = 0usize;
    let mut arr_depth = 0usize;
    let mut saw_root_open = false;
    let mut has_top_level_colon = false;
    let mut root_close_pos = None;

    while i < json.len() {
        let b = json[i];
        if !saw_root_open
            && b != b'{'
            && b != b' '
            && b != b'\t'
            && b != b'\r'
            && b != b'\n'
        {
            return None;
        }
        if b == b'"' {
            let str_start = i + 1;
            i += 1;
            while i < json.len() {
                if json[i] == b'\\' {
                    if i + 1 >= json.len() {
                        return None;
                    }
                    i += 2;
                    continue;
                }
                if json[i] == b'"' {
                    break;
                }
                i += 1;
            }
            if i >= json.len() {
                return None;
            }
            let str_end = i;
            i += 1;
            // If `"manifest_paths"` key is present (e.g. with an invalid non-array value), reject!
            if obj_depth == 1 && arr_depth == 0 && &json[str_start..str_end] == b"manifest_paths" {
                let mut j = i;
                while j < json.len()
                    && (json[j] == b' ' || json[j] == b'\t' || json[j] == b'\r' || json[j] == b'\n')
                {
                    j += 1;
                }
                if j < json.len() && json[j] == b':' {
                    return None;
                }
            }
        } else if b == b'{' {
            if !saw_root_open {
                saw_root_open = true;
            }
            obj_depth += 1;
            i += 1;
        } else if b == b'}' {
            if obj_depth == 0 {
                return None;
            }
            obj_depth -= 1;
            if obj_depth == 0 {
                root_close_pos = Some(i);
            }
            i += 1;
        } else if b == b'[' {
            arr_depth += 1;
            i += 1;
        } else if b == b']' {
            if arr_depth == 0 {
                return None;
            }
            arr_depth -= 1;
            i += 1;
        } else {
            if b == b':' && obj_depth == 1 && arr_depth == 0 {
                has_top_level_colon = true;
            }
            i += 1;
        }
    }

    if !saw_root_open || obj_depth != 0 || arr_depth != 0 {
        return None;
    }
    root_close_pos.map(|pos| (pos, has_top_level_colon))
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

    if let Some((open_b, close_b)) = find_manifest_paths_array_bounds(existing) {
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
                        if i + 1 >= inner.len() {
                            return None;
                        }
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

        return Some(pos);
    }

    // Valid JSON object without a `"manifest_paths"` key yet: insert `"manifest_paths"` before closing `}`
    let (close_brace, has_existing_fields) = find_root_object_insertion_point(existing)?;
    let mut prefix_end = close_brace;
    while prefix_end > 0
        && (existing[prefix_end - 1] == b' '
            || existing[prefix_end - 1] == b'\t'
            || existing[prefix_end - 1] == b'\r'
            || existing[prefix_end - 1] == b'\n')
    {
        prefix_end -= 1;
    }

    if !push_slice_checked(out, &mut pos, &existing[..prefix_end]) {
        return None;
    }
    if has_existing_fields && prefix_end > 0 && existing[prefix_end - 1] != b',' {
        if !push_slice_checked(out, &mut pos, b",") {
            return None;
        }
    }
    if !push_slice_checked(out, &mut pos, b"\n   \"manifest_paths\" : [") {
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
    if !push_slice_checked(out, &mut pos, b"]\n") {
        return None;
    }
    if !push_slice_checked(out, &mut pos, &existing[close_brace..]) {
        return None;
    }

    Some(pos)
}

/// Self-test suite verifying `update_appconfig_manifest_paths` and `parse_windows_cmdline_utf16`
/// against malformed JSON, missing `manifest_paths`, escaped quotes/brackets, and oversized inputs.
pub fn run_self_tests() -> bool {
    let mut out = [0u8; 512];

    // 1. Empty input -> creates valid appconfig.json
    let Some(n1) = update_appconfig_manifest_paths(
        b"",
        b"vr-autoswapaudio.vrmanifest",
        Some(b"C:\\vr\\vr-autoswapaudio.vrmanifest"),
        &mut out,
    ) else {
        return false;
    };
    if find_subslice(&out[..n1], b"\"C:\\\\vr\\\\vr-autoswapaudio.vrmanifest\"").is_none() {
        return false;
    }

    // 2. Missing `manifest_paths` in an existing JSON object with unrelated fields -> preserves unrelated fields and adds `manifest_paths`
    let missing_mp = b"{\n   \"other_setting\" : true,\n   \"nested\" : { \"a\" : [1, 2] }\n}\n";
    let Some(n2) = update_appconfig_manifest_paths(
        missing_mp,
        b"vr-autoswapaudio.vrmanifest",
        Some(b"E:\\vr\\vr-autoswapaudio.vrmanifest"),
        &mut out,
    ) else {
        return false;
    };
    if find_subslice(&out[..n2], b"\"other_setting\" : true").is_none()
        || find_subslice(&out[..n2], b"\"nested\" : { \"a\" : [1, 2] }").is_none()
        || find_subslice(&out[..n2], b"\"manifest_paths\" : [").is_none()
    {
        return false;
    }

    // 3. Escaped quotes & brackets inside strings + unrelated fields before and after `manifest_paths`
    let complex_json = br#"{
   "header" : "val with ] and \"manifest_paths\" : [ \"fake\" ]",
   "manifest_paths" : [
      "C:\\Path\\With \"Quote\" and [Bracket]\\app.vrmanifest",
      "E:\\Old\\vr-autoswapaudio.vrmanifest"
   ],
   "footer" : 42
}"#;
    let Some(n3) = update_appconfig_manifest_paths(
        complex_json,
        b"vr-autoswapaudio.vrmanifest",
        Some(b"E:\\New\\vr-autoswapaudio.vrmanifest"),
        &mut out,
    ) else {
        return false;
    };
    let res3 = &out[..n3];
    if find_subslice(res3, b"\"header\" : \"val with ]").is_none()
        || find_subslice(res3, b"\"footer\" : 42").is_none()
        || find_subslice(res3, b"E:\\\\Old\\\\vr-autoswapaudio.vrmanifest").is_some()
        || find_subslice(res3, b"E:\\\\New\\\\vr-autoswapaudio.vrmanifest").is_none()
        || find_subslice(res3, b"[Bracket]\\\\app.vrmanifest").is_none()
    {
        return false;
    }

    // 4. Malformed JSON (unclosed string, unclosed bracket, non-array manifest_paths, truncated file) MUST return None
    let malformed_cases: &[&[u8]] = &[
        b"{\"manifest_paths\": [\"unterminated}",
        b"{\"manifest_paths\": [\"a\"",
        b"{\"manifest_paths\": 123}",
        b"{\"manifest_paths\": [\"a\"]} }",
        b"not json at all",
    ];
    let mut k = 0;
    while k < malformed_cases.len() {
        if update_appconfig_manifest_paths(
            malformed_cases[k],
            b"vr-autoswapaudio.vrmanifest",
            Some(b"C:\\vr-autoswapaudio.vrmanifest"),
            &mut out,
        )
        .is_some()
        {
            return false;
        }
        k += 1;
    }

    // 5. Oversized output buffer check -> returns None instead of truncating
    let mut tiny_out = [0u8; 20];
    if update_appconfig_manifest_paths(
        complex_json,
        b"vr-autoswapaudio.vrmanifest",
        Some(b"E:\\New\\vr-autoswapaudio.vrmanifest"),
        &mut tiny_out,
    )
    .is_some()
    {
        return false;
    }

    // 6. Windows UTF-16 command-line parser tests (quoted paths with spaces, escaped quotes, backslashes, flags)
    // Input: `"E:\My Path\VR-AutoSwapAudio.exe" --steamvr-launched --output "AC Interface" --input "C:\\Trailing\\\"Quote\"Test"`
    let raw_cmd_ascii = b"\"E:\\My Path\\VR-AutoSwapAudio.exe\" --steamvr-launched --output \"AC Interface\" --input \"C:\\\\Trailing\\\\\\\"Quote\\\"Test\"";
    let mut cmd_u16 = [0u16; 160];
    let mut idx = 0usize;
    while idx < raw_cmd_ascii.len() {
        cmd_u16[idx] = raw_cmd_ascii[idx] as u16;
        idx += 1;
    }
    let mut parsed_args = [
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
    ];
    let argc = parse_windows_cmdline_utf16(&cmd_u16[..raw_cmd_ascii.len()], &mut parsed_args);
    if argc != 5 {
        return false;
    }
    if parsed_args[0].as_bytes() != b"--steamvr-launched"
        || parsed_args[1].as_bytes() != b"--output"
        || parsed_args[2].as_bytes() != b"AC Interface"
        || parsed_args[3].as_bytes() != b"--input"
        || parsed_args[4].as_bytes() != b"C:\\\\Trailing\\\"Quote\"Test"
    {
        return false;
    }

    true
}
