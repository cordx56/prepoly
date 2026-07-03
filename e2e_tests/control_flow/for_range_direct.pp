// `for v in [lo..hi]` iterates the half-open range without materializing it
// as an array. The visible semantics must match the array form: half-open
// bounds, break/continue, empty ranges, and loop-variable reassignment that
// affects only the current iteration's binding.
let total = 0
for i in [0..5] {
    if i == 2 {
        continue
    }
    total += i
}
println(total)

// An empty range never enters the body.
for j in [3..3] {
    println("never")
}

// Bounds are arbitrary expressions, including negatives.
let a = -2
for k in [a..(a + 4)] {
    println(k)
}

// Reassigning the loop variable changes this iteration's binding only;
// iteration order and count are unaffected.
let seen = 0
for m in [0..4] {
    m = m * 10
    seen += m
}
println(seen)

// `break` leaves the loop; the outer range keeps counting independently.
let pairs = 0
for x in [0..3] {
    for y in [0..3] {
        if y == 2 {
            break
        }
        pairs += 1
    }
}
println(pairs)
