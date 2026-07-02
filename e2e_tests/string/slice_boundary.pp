// Pins _string_slice's UTF-8 handling: a byte range that lands mid-character is
// snapped back to the nearest character boundary (matching the interpreter),
// never materializing invalid UTF-8. "héllo" is h(1 byte) é(2 bytes) llo:
//  - [0,2) ends mid-é, snaps to [0,1) = "h";
//  - [2,3) starts mid-é, snaps to [1,3) = "é";
//  - [1,2) both mid-é endpoints collapse to [1,1) = "".
// Also pins that a multibyte slice stays searchable/parseable (defined
// behavior, not UB through from_utf8_unchecked).
fun main() {
    let s = "héllo"
    let a = _string_slice(s, 0, 2)
    println(a)
    println(a.len())
    let b = _string_slice(s, 2, 3)
    println(b)
    println(b.len())
    let c = _string_slice(s, 1, 2)
    println(c.len())
    let f = _string_find(b, "z")
    if f {
        println("found")
    } else {
        println("not found")
    }
}
