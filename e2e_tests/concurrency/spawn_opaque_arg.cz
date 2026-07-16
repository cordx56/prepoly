// Pins: a spawn whose argument is a call result cannot be analyzed for
// ownership; it must be rejected at compile time instead of silently sharing
// its captures with no promotion and no lock.
fun make_task() {
    return () -> { println("x") }
}

fun main() {
    spawn(make_task())
    sync()
}
