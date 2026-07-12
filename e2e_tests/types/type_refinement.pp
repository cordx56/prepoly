// A refinement alias `type Alias = Base { slot: T, .. }` pins a slotted record's
// type parameters, naming a concrete instance. `Counts` is a fully concrete
// `string -> int64` box; a value built by the witness-free constructor and
// annotated `Counts` is pinned to those types -- the bare literals below store as
// `int64` -- so it may be passed where `Counts` is required.
//
// The binding's annotation is what pins the width. Without it the first `put`
// would fix the value type to the literal's default `int32`, and passing that box
// to `sum_values` would be a genuine mismatch: the slot array's element layout
// differs, so it is rejected rather than adapted.

type _Entry = {
    key
    value
}

type Box = {
    key: type
    value: type
    slots_arr: _Entry { key: Self.key, value: Self.value }?[]
    count: int64
}

type Counts = Box {
    key: string,
    value: int64,
}

fun Box.new() {
    let arr = []
    let i: int64 = 0
    while i < 4 {
        arr.push(null)
        i += 1
    }
    return Self { slots_arr: arr, count: 0 }
}

fun Box.put(self, idx, k, v) {
    self.slots_arr[idx] = _Entry { key: k, value: v }
    self.count += 1
}

// Annotated with the refined concrete type.
fun sum_values(c: Counts) -> int64 {
    let total: int64 = 0
    for slot in c.slots_arr {
        if let e = slot {
            total += e.value
        }
    }
    return total
}

fun main() {
    let b: Counts = Box.new()
    b.put(0, "a", 10)
    b.put(1, "b", 32)
    println(sum_values(b))
    println(b.count)
}
