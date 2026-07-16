type Json =
    | JNull
    | JNum { value: int64 }
    | JStr { value: string }
    | JObj { keys: string[], vals: Json[] }

fun Json.get(self, name: string) -> Json! {
    match self {
        Json.JObj { keys, vals } => {
            for i in [0..keys.len()] {
                if keys[i] == name { return vals[i] }
            }
            return error("missing field '{name}'")
        }
        _ => {}
    }
    return error("not an object")
}

fun Json.into(self) -> infer! {
    match self {
        Json.JNum { value } => { return infer.from(value) }
        Json.JStr { value } => { return infer.from(value) }
        Json.JNull => { return null }
        Json.JObj { keys, vals } => {
            let ret: infer
            for field in fields(ret) {
                ret[field] = self.get(field)!.into()!
            }
            return ret
        }
    }
}

type Point = { x: int64, y: int64 }

fun show(j: Json) {
    const decoded: Point! = j.into()
    match decoded {
        Ok { value } => { println("ok: {value.x} {value.y}") }
        Err { error } => { println("err: {error}") }
    }
}

fun main() {
    show(Json.JObj { keys: ["x", "y"], vals: [Json.JNum { value: 1 }, Json.JNum { value: 2 }] })
    show(Json.JObj { keys: ["x"], vals: [Json.JNum { value: 1 }] })
    show(Json.JNum { value: 5 })
    show(Json.JObj { keys: ["x", "y"], vals: [Json.JStr { value: "no" }, Json.JNum { value: 2 }] })
}
