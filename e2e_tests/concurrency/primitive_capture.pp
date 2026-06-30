// A spawn closure captures a primitive `tag` (read) and a record `result`
// (mutated). A primitive is copied by value across the spawn boundary -- it has no
// heap object to share -- so it must not be promoted to a cown or wrapped in a
// lock; doing so previously fed a scalar to the runtime's pointer-typed promotion
// and failed compilation. This pins that a primitive capture alongside a real
// cown capture compiles and runs: only the parent prints (after the join), so the
// output is deterministic.
type Box = {
    value: int64
    set(self, n: int64) { self.value = n }
}

fun main() {
    let tag = 7
    let result = Box { value: 0 }
    spawn(() -> {
        result.set(tag * 6)
    })
    sync()
    println("result = {result.value}")
}
