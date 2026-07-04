type Json =
    | JNull
    | JNum { value: int64 }
    | JStr { value: string }
    | JObj { keys: string[], vals: Json[] }

fun Json.get(self, name: string) -> Json! {
    match self {
        Json.JObj { keys, vals } => {
            for i in [0..keys.len()] {
                if keys[i] == name {
                    return vals[i]
                }
            }
            return error("missing field '{name}'")
        }
        _ => {}
    }
    return error("not an object")
}

// The generic reflective decoder: leaf keys convert via infer.from; a record
// key walks its own fields, decoding each recursively by the field's type.
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

type Address = { city: string, zip: int64 }
type User = { name: string, age: int64, address: Address }

fun main() {
    const addr = Json.JObj {
        keys: ["city", "zip"],
        vals: [Json.JStr { value: "Tokyo" }, Json.JNum { value: 100 }],
    }
    const obj = Json.JObj {
        keys: ["name", "age", "address"],
        vals: [Json.JStr { value: "Aki" }, Json.JNum { value: 30 }, addr],
    }
    const user: User = obj.into()!
    println("{user.name}, {user.age}, {user.address.city} {user.address.zip}")
}
