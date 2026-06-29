// float32 conversions must be truncated to 32-bit precision in the JIT, not stored
// as a raw f64 bit pattern -- otherwise every float32 reads as garbage. The output
// must match the interpreter byte-for-byte.
fun main() {
    let a = float32.from(1)
    let b = float32.from(3)
    println(a / b)
    println(float32.from(10))
    println(float32.parse("2.5")!)
}
