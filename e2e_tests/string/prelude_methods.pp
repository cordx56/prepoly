// The prelude string methods end to end: trim, case mapping, prefix/suffix
// tests, find, replace, split + join. Split edge cases: an empty separator is
// "no split" (one whole-string field), an empty subject yields one empty field
// (rendered `[]`, so its length is printed), and interior empty fields are kept.
fun main() {
    println("  padded\t\n".trim())
    println("MiXeD".to_upper())
    println("MiXeD".to_lower())
    println("prefix-rest".starts_with("prefix"))
    println("prefix-rest".ends_with("rest"))
    println("prefix-rest".starts_with("rest"))
    let pos = "hello".find("ll")
    if pos {
        println(pos)
    }
    println("hello".find("zz") == null)
    println("a-b-a".replace("a", "xy"))
    println("a,b,c".split(","))
    println("a,,b".split(","))
    println("".split(",").len())
    println("abc".split(""))
    println(["x", "y", "z"].join(", "))
}
