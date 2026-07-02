// Pins: the task reads `h.inner` while the spawner REPLACES the inner record
// each iteration, releasing the old one. With the capture frozen the reader
// could hold a freed record (use after free); cowning `h` serializes the swap
// against the read.
type Inner = {
    value: int64
}
type Holder = {
    inner: Inner
}

fun main() {
    let h = Holder { inner: Inner { value: 0 } }
    spawn(() -> {
        let i = 0
        let sum = 0
        while i < 200000 {
            sum = sum + h.inner.value
            i = i + 1
        }
        println("reader done")
    })
    let j = 0
    while j < 200000 {
        h.inner = Inner { value: j }
        j = j + 1
    }
    sync()
    println("value = {h.inner.value}")
}
