// Pins that the engine-symbol -> LLVM-name mapping is injective: `int32?[]`
// (slice of nullable) and `int32[]?` (nullable slice) instantiate `probe`
// twice, and the two instances must get distinct LLVM functions. The old
// sanitize folded every punctuation char to `_`, merging the two names and
// SIGSEGV-ing the JIT.
fun probe(x) -> int64 {
    if x {
        return x.len()
    }
    return -1
}

fun main() {
    let a: int32?[] = [1, 2, 3]
    let b: int32[]? = [4, 5]
    println(probe(a))
    println(probe(b))
}
