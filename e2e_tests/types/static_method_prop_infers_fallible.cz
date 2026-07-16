// A `!` on a USER TYPE'S STATIC method must make an unannotated caller fallible,
// exactly as a `!` on a free function does. The light pass has no receiver to
// infer for `Conn.open(..)` (the base names a type, not a value), so it looks the
// return up under the type's own name; without that the call's type stayed
// unknown, `use_it` inferred non-fallible, and `use_it(..)!` failed to compile.
type Conn = { _id: int64 }

fun Conn.open(id: int64) -> Conn! {
    if id < 0 {
        return error("bad id")
    }
    return Conn { _id: id }
}

fun Conn.id(self) -> int64 {
    return self._id
}

// Unannotated on purpose: its fallibility must be inferred from the `!` below.
fun use_it(id: int64) {
    let c = Conn.open(id)!
    return c.id()
}

fun main() {
    println(use_it(7)!)
    match use_it(-1) {
        Ok { value } => println("unexpected"),
        Err { error } => println("err: {error}"),
    }
}
