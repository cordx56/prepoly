// An unannotated (dynamic) variant field, once constructed, is matched and used at
// its real per-value type: the value's substitution records the field type so the
// match binding is concrete (int32 here, string there).
type Wrapper =
    | Wrap { value }

fun main() {
    const a = Wrapper.Wrap { value: 41 }
    const n: int32 = match a { Wrap { value } => value + 1 }
    println("{n}")

    const b = Wrapper.Wrap { value: "text" }
    match b {
        Wrap { value } => { println("{value}") }
    }
}
