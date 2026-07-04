type P = { x: int64, y: int64 }
fun main() {
    const p = P { x: 1, y: 2 }
    for f in fields(p) {
        let f = 3
        println(f)
    }
}
