// Standard array utilities, written in Prepoly on top of the runtime
// primitives and implemented as methods on the array type with `fun infer[].m`.
// The receiver is `self`, so `arr.map(f)` dispatches here. These methods are
// part of the implicit prelude and are available wherever an array is, with no
// import. `infer[]` requires an array while leaving the element type to
// inference, so each method stays generic over it.

// Apply `f` to each element, returning a new array of the results.
fun infer[].map(self, f) {
    let result = []
    for item in self {
        result.push(f(item))
    }
    return result
}

// Keep the elements for which `pred` returns true.
fun infer[].filter(self, pred) {
    let result = []
    for item in self {
        if pred(item) {
            result.push(item)
        }
    }
    return result
}

// Left fold: combine elements into a single accumulator starting from `init`.
fun infer[].fold(self, init, f) {
    let acc = init
    for item in self {
        acc = f(acc, item)
    }
    return acc
}

// Run `f` for its side effects on each element.
fun infer[].each(self, f) {
    for item in self {
        f(item)
    }
}

// A copy of `self[start..end]`. Bounds are `int64` (the type of `len`), so both
// `arr.slice(1, 4)` and `arr.slice(1, arr.len())` work and the loop counter stays
// `int64`.
fun infer[].slice(self, start: int64, end: int64) {
    let one: int64 = 1
    let result = []
    let i = start
    while i < end {
        result.push(self[i])
        i += one
    }
    return result
}

// The elements of `self` in reverse order.
fun infer[].reverse(self) {
    let one: int64 = 1
    let result = []
    let i = len(self) - one
    while i >= 0 {
        result.push(self[i])
        i -= one
    }
    return result
}

// Membership test for arrays: `arr.contains(x)` checks elements by `==`.
// Substring search on strings is a distinct operation (`s.find(sub)`), because
// strings are not iterated directly.
fun infer[].contains(self, x) {
    for item in self {
        if item == x {
            return true
        }
    }
    return false
}

// Insertion sort returning a new ascending array, ordering with `<`/`>`.
fun infer[].sort(self) {
    let one: int64 = 1
    let result = []
    for item in self {
        result.push(item)
    }
    let i: int64 = one
    while i < len(result) {
        let key = result[i]
        let j = i - one
        while j >= 0 && result[j] > key {
            result[j + one] = result[j]
            j -= one
        }
        result[j + one] = key
        i += one
    }
    return result
}
