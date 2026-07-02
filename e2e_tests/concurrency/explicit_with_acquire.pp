// Explicit `with(cown, f)` acquisition: two spawned tasks mutate a shared
// record inside `with` scopes, and the parent reads the final state through
// `with` after `sync()`. The total is deterministic because every access is
// serialized by the cown lock.
type Tally = {
    total: int64
}

fun Tally.add(self, n: int64) { self.total = self.total + n }

fun main() {
    let tally = Tally { total: 0 }
    spawn(() -> {
        with(tally, (t) -> {
            let i = 0
            while i < 1000 {
                t.add(2)
                i = i + 1
            }
        })
    })
    spawn(() -> {
        with(tally, (t) -> {
            let i = 0
            while i < 1000 {
                t.add(3)
                i = i + 1
            }
        })
    })
    sync()
    with(tally, (t) -> {
        println("total = {t.total}")
    })
}
