// Pins a non-void function whose only returns are inside `while true`: the MIR
// builder synthesizes a fall-through `Return(void)` for the unterminated final
// block, which the back end used to emit as a mistyped `ret i1 false` (an LLVM
// verifier failure). The fall-through is unreachable and must lower to a trap.
fun f() -> int64 {
    let i = 0
    while true {
        i += 1
        if i > 3 {
            return i
        }
    }
}

fun classify(n: int64) -> string {
    while true {
        if n > 0 {
            return "positive"
        }
        if n < 0 {
            return "negative"
        }
        return "zero"
    }
}

fun main() {
    println(f())
    println(classify(7))
    println(classify(-1))
    println(classify(0))
}
