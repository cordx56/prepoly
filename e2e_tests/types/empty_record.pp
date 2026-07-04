// A fieldless record type is inside the typed subset: its (empty)
// substitution must read as supported everywhere a type is checked --
// as a return type, a parameter, a global, and a record field.
type Empty = {}
type Holder = {
  e: Empty
}

fun mk() -> Empty {
  return Empty {}
}

fun take(e: Empty) -> Holder {
  return Holder { e: e }
}

const g = mk()

fun main() {
  let e = mk()
  let h = take(e)
  let g2 = take(g)
  println("ok")
}
