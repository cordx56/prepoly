// Standard array utilities, written in Prepoly on top of the runtime
// primitives. The receiver is the first parameter so `arr.map(f)` resolves
// here through UFCS (DESIGN.md 9.4). These functions are part of the implicit
// prelude and need no import.

// Apply `f` to each element, returning a new array of the results. `infer[]`
// requires an array while leaving the element type to inference, so `map` stays
// generic over it.
fun map(arr: infer[], f) {
    let result = []
    for item in arr {
        result.push(f(item))
    }
    return result
}

// Keep the elements for which `pred` returns true.
fun filter(arr: infer[], pred) {
    let result = []
    for item in arr {
        if pred(item) {
            result.push(item)
        }
    }
    return result
}

// Left fold: combine elements into a single accumulator starting from `init`.
fun fold(arr: infer[], init, f) {
    let acc = init
    for item in arr {
        acc = f(acc, item)
    }
    return acc
}

// Run `f` for its side effects on each element.
fun each(arr: infer[], f) {
    for item in arr {
        f(item)
    }
}

// A copy of `arr[start..end]`. Bounds are `int64` (the type of `len`), so both
// `arr.slice(1, 4)` and `arr.slice(1, arr.len())` work and the loop counter stays
// `int64` (Prepoly has no implicit conversion between integer widths).
fun slice(arr: infer[], start: int64, end: int64) {
    let one: int64 = 1
    let result = []
    let i = start
    while i < end {
        result.push(arr[i])
        i += one
    }
    return result
}

// The elements of `arr` in reverse order.
fun reverse(arr: infer[]) {
    let one: int64 = 1
    let result = []
    let i = len(arr) - one
    while i >= 0 {
        result.push(arr[i])
        i -= one
    }
    return result
}

// Membership test. Polymorphic over arrays and strings so that both
// `coll.contains(x)`: membership of element `x` in a sequence by `==`. Polymorphic
// over any iterable of comparable elements -- the solver infers the element type
// per call, so it needs no annotation. Substring search on strings is a distinct
// operation (`_string_find`), not this element test.
fun contains(coll, x) {
    for item in coll {
        if item == x {
            return true
        }
    }
    return false
}

// Insertion sort returning a new ascending array, ordering with `<`/`>`.
fun sort(arr: infer[]) {
    let one: int64 = 1
    let result = []
    for item in arr {
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
