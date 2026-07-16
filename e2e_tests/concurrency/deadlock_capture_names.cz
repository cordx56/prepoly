// Pins: two spawns capturing the SAME two objects under names that sort in
// opposite orders (a/b vs b/z where z aliases a). The old auto-wrap acquired
// nested `with` locks in capture-NAME order, so the two bodies deadlocked
// against each other. The group wrap (_with_all) acquires by runtime ADDRESS
// order, so all 100 tasks complete and the totals are exact.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let a = Counter { value: 0 }
    let b = Counter { value: 0 }
    let z = a
    let k = 0
    while k < 50 {
        spawn(() -> {
            a.add(1)
            b.add(1)
        })
        spawn(() -> {
            b.add(1)
            z.add(1)
        })
        k = k + 1
    }
    sync()
    println("a = {a.value} b = {b.value}")
}
