type P = { x: int64, y: string }
fun main() {
    const p = P { x: 1, y: "a" }
    for f in fields(p) {
        println(p[f] + 1)
    }
}
