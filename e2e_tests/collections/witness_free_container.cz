// A witness-free generic container: `new()` builds an empty array whose element
// type is never declared and never seeded with a sample. The element type is
// fixed only by later use -- `add(k, v)` pushes a `Pair`, so the checker resolves
// the instance and the back end follows it. Two differently-typed boxes in one
// program (`string`/`int32` keys and values swapped) must each compile to their
// own instance, exercising the result-type keying of a no-argument constructor.

type Pair = {
    k
    v
}

type Box = {
    items
}

fun Box.new() {
    let items = []
    return Self { items: items }
}

fun Box.add(self, k, v) {
    self.items.push(Pair { k: k, v: v })
}

fun Box.get_k(self, i) {
    return self.items[i].k
}

fun Box.get_v(self, i) {
    return self.items[i].v
}

fun main() {
    let a = Box.new()
    a.add("x", 10)
    a.add("y", 20)
    println("a: {a.get_k(0)}={a.get_v(0)} {a.get_k(1)}={a.get_v(1)}")

    let b = Box.new()
    b.add(7, "seven")
    println("b: {b.get_k(0)}={b.get_v(0)}")
}
