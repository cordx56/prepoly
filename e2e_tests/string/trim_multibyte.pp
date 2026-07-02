// Pins trim on strings whose edges are multibyte UTF-8 characters. The trailing
// probe `_string_char_at(s, end - 1)` lands mid-character there and yields null;
// that null must (a) not reach string `==` as a dereference (the JIT used to
// fault) and (b) be treated as "not whitespace" so the character is preserved.
// ASCII whitespace around multibyte text must still strip on both sides.
fun main() {
    println(" héllo ".trim())
    println("héé".trim())
    println("あいう".trim())
    println("  あ  ".trim())
    println("\t é \n".trim())
    println("".trim())
    println("   ".trim().len())
}
