// A structural SUPERtype element must not be accepted by `push`: storing an
// `Animal` (fewer fields) into a `Dog[]` and reading it back as a `Dog` would
// read past the `Animal` layout in the unboxed back end. Must be rejected.
type Animal = {
    name: string
}

type Dog = {
    name: string
    breed: string
}

fun main() {
    let d = Dog { name: "Rex", breed: "lab" }
    let xs: Dog[] = [d]
    xs.push(Animal { name: "generic" })
}
