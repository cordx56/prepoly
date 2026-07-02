// Pins two free-variable analysis rules for closures:
// - a name used only inside a range expression (`[0..n]`) is still captured
//   (the sub-expression walk must descend into ranges);
// - a read *before* a same-named later `let` refers to the outer binding and
//   is captured (binding collection must be ordered, not body-flat).
// Both used to fail with "outside the typed subset" (missing capture).
fun main() {
    let n = 5
    let f = () -> {
        let r = [0..n]
        return r.len()
    }
    println(f())

    let y = 10
    let g = () -> {
        println(y)
        let y = 5
        println(y)
    }
    g()
}
