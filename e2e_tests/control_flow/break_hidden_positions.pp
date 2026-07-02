// Pins that `break` is found wherever it hides:
//  - a break in a call argument or a let value makes a `while true` breakable,
//    so a `-> int32` function ending in that loop can fall through and must be
//    rejected (previously judged infinite -> garbage return value);
//  - a break outside any loop hidden in a call argument is rejected.
fun f(c: bool) -> int32 {
    while true {
        println(if c { break } else { 1 })
    }
}

fun g(c: bool) -> int32 {
    while true {
        let x = if c { break } else { 0 }
        println(x)
    }
}

fun h(c: bool) {
    println(if c { break } else { 0 })
}

println(f(true))
