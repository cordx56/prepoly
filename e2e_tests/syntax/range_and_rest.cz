// `[n..m]` builds the half-open integer range `[n, n+1, ..., m-1]`.
println([1..5])
println([0..0])
let n = 3
println([n..(n + 3)])

// `T.A { field, .. }` binds some fields of a variant and omits the rest with `..`.
type Holder =
    | Full { data: int32, tag: int32 }
    | Empty

fun first(h: Holder) -> int32 {
    return match h {
        Holder.Full { data, .. } => data,
        Holder.Empty => 0,
    }
}

println(first(Holder.Full { data: 42, tag: 7 }))
println(first(Holder.Empty))
