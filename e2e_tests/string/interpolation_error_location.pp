// A type error inside a string interpolation is attributed to this file and
// the fragment's real position, not to the first stdlib entry of the source
// map (the fragment is re-lexed from zero and its spans shifted back).
fun main() {
    println("value: {undefined_var}")
}
