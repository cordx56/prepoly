// The structural graceful-degradation probe folds an `if <field>` arm only when
// the arm's straight-line return value kind-conflicts with the declared return
// type (the exact rule the back end prunes by). An unrelated type error inside
// the guarded arm must still be reported -- the arm executes at runtime.
type T = { flag: bool }

fun main() {
    let a = [1]
    let t = T { flag: true }
    if t.flag {
        a.push("boom")
    }
    println(a[1] + 1)
}
