type User = {
    name: string,
    age: int32,
    score: float64,
}

fun User.render(self) -> string {
    let out = ""
    for field in fields(self) {
        out = out + "{field}={self[field]};"
    }
    return out
}

// Literal adaptation must happen per copy: `zero` flows into an int8 field in
// one copy and an int64 field in another.
type Counters = {
    small: int8,
    big: int64,
}

fun zeroed() -> Counters {
    let ret: Counters
    for f in fields(ret) {
        ret[f] = 1
    }
    return ret
}

fun main() {
    const u = User { name: "aki", age: 20, score: 88.5 }
    println(u.render())
    println(zeroed())
    const c = zeroed()
    println(c.small + 1)
}
