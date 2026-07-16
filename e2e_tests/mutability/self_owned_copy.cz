// `self` is normally a reference; annotating it `self: Self` makes the method
// work on an owned deep copy instead (types.md: "To work on an owned copy of
// self instead, annotate it `self: Self`"). Mutations stay in the copy.
type Counter = {
    n: int32
}

fun Counter.bump_copy(self: Self) -> int32 {
    self.n += 1
    return self.n
}

fun Counter.bump(self) {
    self.n += 1
}

fun main() {
    let c = Counter { n: 10 }
    println(c.bump_copy())
    println(c.n)
    c.bump()
    println(c.n)
}
