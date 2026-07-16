// Pins that a method call is not padded from a same-named free function's
// signature: `describe` exists both as a free function with a trailing
// nullable parameter and as a Box method. `b.describe()` must call the method
// with its own arity (the old lowering pushed a `null` from the free
// function's signature and the call was rejected as outside the typed subset).
type Box = { v: int32 }

fun describe(a: int32, extra: string?) -> string {
    return "free"
}

fun Box.describe(self) -> string {
    return "method {self.v}"
}

fun main() {
    let b = Box { v: 7 }
    println(b.describe())
    println(describe(1))
}
