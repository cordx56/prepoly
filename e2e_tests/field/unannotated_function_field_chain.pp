// Unannotated function-typed fields: closures stored at construction take
// their per-instance types from the literal (not the shared declaration
// variable), a method building a new instance from captured fields types its
// wrapper closure from its own body, and chained instances with DIFFERENT
// closure types per call site each monomorphize correctly.
type Iter = {
    start,
    next,
    trans,
}

fun Iter.map_lazy(self, func) {
    const g = self.trans
    return Iter {
        start: self.start,
        next: self.next,
        trans: (x) -> func(g(x)),
    }
}

fun Iter.collect(self) {
    let ans = []
    const n = self.next
    const t = self.trans
    while 1 {
        const term = n(self.start)
        if term {
            self.start = term
            ans.push(t(term))
        } else {
            break
        }
    }
    return ans
}

fun _range_next(start: int32, stop: int32, step: int32) {
    if start == stop { return null }
    return start + step
}

fun range(start, stop, step) {
    return Iter {
        start: start,
        next: (x: int32) -> _range_next(x, stop, step),
        trans: (x: int32) -> x,
    }
}

fun main() {
    let doubled = range(0, 5, 1)
        .map_lazy((x: int32) -> 2 * x)
        .map_lazy((x: int32) -> x - 1)
        .collect()
    println(doubled)
}
