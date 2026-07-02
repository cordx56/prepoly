// Numeric flow conversions allow int->int, float->float, and int->float, but
// float->int must be explicit (types.md: "The one exception is float to int").
fun main() {
    let x: int32 = 1.5
    println(x)
}
