// A parameter used as a method RECEIVER is view-ineligible: dispatch needs the
// full value, so the argument keeps today's per-concrete-type path and the
// structural method resolution still sees every field of the caller's value.
type Person = {
    name: string
}

fun Person.hello(self) {
    println("I am {self.name}")
}

fun call_hello(p) {
    p.hello()
}

fun main() {
    call_hello({ name: "Asimov", age: 72 })
}
