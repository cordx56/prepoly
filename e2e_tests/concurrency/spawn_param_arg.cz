// Pins: spawning a closure received as a PARAMETER is rejected: the closure's
// captures belong to another function's scope, so this function cannot promote
// or guard them. Compiling it silently was an unguarded cross-thread share.
fun run(f) {
    spawn(f)
}

fun main() {
    run(() -> { println("x") })
    sync()
}
