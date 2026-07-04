type Json =
    | JNull
    | JNum { value: int64 }
    | JStr { value: string }

fun Json.into(self) -> infer! {
    match self {
        Json.JNum { value } => { return infer.from(value) }
        Json.JStr { value } => { return infer.from(value) }
        Json.JNull => { return null }
        _ => {}
    }
    return error("no conversion")
}

fun main() {
    const n: int64 = Json.JNum { value: 42 }.into()!
    println(n)
    const s: string = Json.JStr { value: "hi" }.into()!
    println(s)
    const bad: int64! = Json.JStr { value: "x" }.into()
    match bad {
        Ok { value } => { println(value) }
        Err { error } => { println("err: {error}") }
    }
}
