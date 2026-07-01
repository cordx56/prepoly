// A witness-free open-addressing-style container: `new()` pre-sizes a slot array
// with `null` (no sample value needed), `put` stores a non-null record at a
// computed index, and reads narrow the nullable slot with `if let`. The element
// type is fixed entirely by use -- the back end follows the checker's resolved
// instance through the slot array's nullable element.

type Pair = {
    k
    v
}

type Slots = {
    items: infer?[]
}

fun Slots.new() {
    let items = []
    let i: int64 = 0
    while i < 4 {
        items.push(null)
        i += 1
    }
    return Self { items: items }
}

fun Slots.put(self, i: int64, k, v) {
    self.items[i] = Pair { k: k, v: v }
}

fun Slots.value_at(self, i: int64) {
    if let e = self.items[i] {
        return e.v
    }
    return null
}

fun main() {
    let s = Slots.new()
    s.put(0, "a", 10)
    s.put(3, "d", 40)
    println("at0={s.value_at(0)} at3={s.value_at(3)} at1={s.value_at(1)}")
}
