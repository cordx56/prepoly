// `infer[]` is an array whose element type is inferred, so the function is
// generic over the element type (each call instantiates independently).
fun head(xs: infer[]) {
    return xs[0]
}
fun count(xs: infer[]) {
    return xs.len()
}

println(head([10, 20, 30]))
println(head(["a", "b"]))
println(count([1, 2, 3, 4, 5]))
