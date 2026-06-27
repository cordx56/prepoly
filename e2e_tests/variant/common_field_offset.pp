type Thing =
    | A { x: int64, tag: int32 }
    | B { tag: int32, y: int64 }

fun tag_of(t: Thing) -> int32 {
    return t.tag
}

fun main() {
    println("{tag_of(Thing.A { x: 100, tag: 1 })}")
    println("{tag_of(Thing.B { tag: 2, y: 200 })}")
}
