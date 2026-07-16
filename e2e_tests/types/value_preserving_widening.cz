// The widenings that stay implicit: same-signed wider int, unsigned into a
// strictly wider signed int, exact int -> float, float32 -> float64, and the
// operator common type over the same lattice.
fun main() {
    let a: int32 = -5
    let b: int64 = a
    let u: uint8 = 200
    let c: int32 = u
    let f: float64 = a
    let s: float32 = 1.5
    let d: float64 = s
    println("{b} {c} {f} {d} {u + a}")
}
