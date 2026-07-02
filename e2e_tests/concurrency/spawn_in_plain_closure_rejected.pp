// A spawn inside a closure that is never itself spawned is invisible to the
// ownership pass (no promotion, no guarding), so it is rejected outright.
type Tally = { total: int64 }
fun Tally.add(self, n: int64) { self.total = self.total + n }

fun main() {
    let tally = Tally { total: 0 }
    let f = () -> {
        spawn(() -> {
            tally.add(1)
        })
    }
    f()
    sync()
    println(tally.total)
}
