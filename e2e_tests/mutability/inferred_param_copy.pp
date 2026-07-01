// An unannotated parameter's passing mode is inferred from the body: a parameter
// the body mutates is a private deep copy (the caller's value is unchanged), an
// unmutated one is a shared borrow. This holds for both free functions and
// methods. Explicit `ref(mut(T))` (see mut_param.pp) is the way to opt into
// write-through instead.
type Bag = {
    items: int32[]
}

// `extra` is unannotated and mutated, so it is deep-copied on entry: pushing to it
// never touches the caller's array, and the copy is what the field stores.
fun Bag.fill(self, extra) {
    extra.push(0)
    self.items = extra
}

// `xs` is unannotated and mutated: a private copy, so `a` in main stays unchanged.
fun bump(xs) {
    xs.push(99)
    println(xs)
}

// `ys` is mutated via the loop variable (`e *= 2` writes back into the array),
// which is inferred as a copy just like a direct `ys[i] = ..` store.
fun scale(ys) {
    for e in ys {
        e *= 2
    }
    println(ys)
}

fun main() {
    let a = [1, 2, 3]
    bump(a)
    println(a)

    let b = [1, 2, 3]
    scale(b)
    println(b)

    let bag = Bag { items: [7] }
    let xs = [1, 2]
    bag.fill(xs)
    println(xs)
    println(bag.items)
}
