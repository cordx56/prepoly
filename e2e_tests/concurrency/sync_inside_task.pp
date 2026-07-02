// Pins: sync() inside a spawned task must skip the task's OWN handle in the
// thread registry. Joining it is a self-join: pthread_join(self) fails with
// EDEADLK and the std join panicked, aborting the whole program (the spawner's
// long loop delays its own sync so the task's handle is still registered).
// After the fix the task's sync returns and main's sync still reaps the task,
// so the output order is deterministic.
type Box = {
    value: int64
}

fun main() {
    spawn(() -> {
        sync()
        println("task done")
    })
    let b = Box { value: 0 }
    let j = 0
    while j < 20000000 {
        b.value = b.value + 1
        j = j + 1
    }
    sync()
    println("main done {b.value}")
}
