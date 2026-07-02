// A spawned task may not touch a module global that is written anywhere:
// module storage has no binding to promote to a cown, so the task and the
// writer would race unguarded. Never-written globals stay shareable.
type Tally = { total: int64 }
fun Tally.add(self, n: int64) { self.total = self.total + n }

let g = Tally { total: 0 }

fun main() {
    spawn(() -> {
        g.add(1)
    })
    g.add(1)
    sync()
    println(g.total)
}
