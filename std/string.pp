// Standard string utilities built on the `_string_*` runtime primitives, exposed
// as methods on `string` with `fun string.m`. All string indices are UTF-8 byte
// offsets: `len`, slicing, `find`, indexing, and `_string_char_at` agree on byte
// positions, and the per-character helpers advance by each character's byte
// length. Part of the implicit prelude.

// Split `self` on every occurrence of `sep`.
fun string.split(self, sep: string) -> string[] {
    let result = []
    // An empty separator has a match at every position, so `_string_find` always
    // returns 0 and `start` never advances -- an infinite loop. Treat it as no
    // split (the whole string), mirroring `replace`'s empty-`old` guard.
    if len(sep) == 0 {
        result.push(self)
        return result
    }
    let start: int64 = 0
    while start < len(self) {
        let rest = _string_slice(self, start, len(self))
        let pos = _string_find(rest, sep)
        if pos != null {
            result.push(_string_slice(self, start, start + pos))
            start = start + pos + len(sep)
        } else {
            result.push(_string_slice(self, start, len(self)))
            start = len(self)
        }
    }
    if len(self) == 0 {
        result.push("")
    }
    return result
}

// Strip leading and trailing ASCII whitespace.
fun string.trim(self) -> string {
    let one: int64 = 1
    let start: int64 = 0
    let end = len(self)
    while start < end {
        let c = _string_char_at(self, start)
        if c == " " || c == "\t" || c == "\n" || c == "\r" {
            start += one
        } else {
            break
        }
    }
    while end > start {
        let c = _string_char_at(self, end - one)
        if c == " " || c == "\t" || c == "\n" || c == "\r" {
            end -= one
        } else {
            break
        }
    }
    return _string_slice(self, start, end)
}

fun string.starts_with(self, prefix: string) -> bool {
    if len(prefix) > len(self) {
        return false
    }
    return _string_slice(self, 0, len(prefix)) == prefix
}

fun string.ends_with(self, suffix: string) -> bool {
    if len(suffix) > len(self) {
        return false
    }
    return _string_slice(self, len(self) - len(suffix), len(self)) == suffix
}

// `s.find(sub)`: the byte offset of the first occurrence of substring `sub` in
// `s`, or null if absent. This is the string substring search, distinct from the
// polymorphic element-membership `contains` (use `s.find(sub) != null` for a
// substring test).
fun string.find(self, sub: string) -> int64? {
    return _string_find(self, sub)
}

// Replace every occurrence of `old` with `new`.
fun string.replace(self, old: string, new: string) -> string {
    if len(old) == 0 {
        return self
    }
    let result = ""
    let start: int64 = 0
    while start < len(self) {
        let rest = _string_slice(self, start, len(self))
        let pos = _string_find(rest, old)
        if pos != null {
            result = result + _string_slice(self, start, start + pos) + new
            start = start + pos + len(old)
        } else {
            result = result + _string_slice(self, start, len(self))
            start = len(self)
        }
    }
    return result
}

// The characters of `self` as a one-element-per-character array. Advances by each
// character's byte length so multibyte UTF-8 characters are handled correctly.
fun string.chars(self) -> string[] {
    let result = []
    let i: int64 = 0
    while i < len(self) {
        if let c = _string_char_at(self, i) {
            result.push(c)
            i += len(c)
        } else {
            break
        }
    }
    return result
}

// Join the string elements of `self` with `sep` between each. A method on the
// array type, so `parts.join(", ")` reaches it.
fun string[].join(self, sep: string) -> string {
    let result = ""
    let first = true
    for p in self {
        if first {
            result = result + p
            first = false
        } else {
            result = result + sep + p
        }
    }
    return result
}

// ASCII upper-casing implemented over the UTF-8 byte view. An ASCII case change
// preserves UTF-8 validity, so the byte->string conversion cannot fail; matching
// (rather than `!`) keeps `to_upper` non-fallible, returning `string` not `string!`.
fun string.to_upper(self) -> string {
    let bytes = _string_bytes(self)
    let result = []
    for b in bytes {
        if b >= 97 && b <= 122 {
            result.push(b - 32)
        } else {
            result.push(b)
        }
    }
    return match _string_from_bytes(result) {
        Ok { value } => value,
        Err { error } => self,
    }
}

fun string.to_lower(self) -> string {
    let bytes = _string_bytes(self)
    let result = []
    for b in bytes {
        if b >= 65 && b <= 90 {
            result.push(b + 32)
        } else {
            result.push(b)
        }
    }
    return match _string_from_bytes(result) {
        Ok { value } => value,
        Err { error } => self,
    }
}
