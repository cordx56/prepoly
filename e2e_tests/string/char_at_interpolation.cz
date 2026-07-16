// Interpolating the nullable string `_string_char_at` returns must not double-free
// the cell's string: on the non-null path its `to_string` is the identity, an alias
// to the cell's owned string. Before the fix this corrupted the JIT heap.
fun main() {
    let c = _string_char_at("ab", 1)
    println("got={c}")
    let miss = _string_char_at("ab", 9)
    println("miss={miss}")
}
