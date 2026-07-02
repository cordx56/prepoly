// Pins: a capture NOBODY ever mutates keeps the freeze (immutable share) path:
// the task and the spawner both read it lock-free and the program still runs
// deterministically. Guards that the writer-upgrade rule does not cown every
// read-only capture.
type Pair = {
    x: int64,
    y: int64
}

fun main() {
    let p = Pair { x: 2, y: 3 }
    spawn(() -> {
        println("task sum = {p.x + p.y}")
    })
    sync()
    println("main sum = {p.x + p.y}")
}
