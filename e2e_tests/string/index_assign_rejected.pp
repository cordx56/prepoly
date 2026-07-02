// Strings are immutable: assignment through a string index is a compile error
// rather than an unchecked store into non-existent element storage.
fun main() {
    let s = "abc"
    s[0] = "z"
    println(s)
}
