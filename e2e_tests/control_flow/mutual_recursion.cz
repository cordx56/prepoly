// Mutual recursion on the typed backend: a call reaching back into an
// in-progress instance types against its annotated (authoritative) return
// type instead of being rejected. Pins all three callable kinds: free
// functions, methods, and fallible functions (whose `T!` annotation fixes the
// Ok payload; the error payload is the guessed-and-validated `string`), plus
// walkers over a recursive record type -- the shape a JSON decoder needs.
fun is_even(n: int64) -> bool {
    if n == 0 {
        return true
    }
    return is_odd(n - 1)
}

fun is_odd(n: int64) -> bool {
    if n == 0 {
        return false
    }
    return is_even(n - 1)
}

type Node = {
    value: int64,
    next: Node?,
}

fun sum_even(n: Node?) -> int64 {
    if let node = n {
        return node.value + sum_odd(node.next)
    }
    return 0
}

fun sum_odd(n: Node?) -> int64 {
    if let node = n {
        return sum_even(node.next)
    }
    return 0
}

type Tree = {
    label: string,
}

fun Tree.ping(self, n: int64) -> string {
    if n == 0 {
        return self.label
    }
    return self.pong(n - 1)
}

fun Tree.pong(self, n: int64) -> string {
    return self.ping(n)
}

fun descend_a(i: int64) -> int64! {
    if i >= 3 {
        return error("too deep")
    }
    if i == 2 {
        return i
    }
    return descend_b(i + 1)!
}

fun descend_b(i: int64) -> int64! {
    return descend_a(i)!
}

fun main() {
    println(is_even(10))
    println(is_odd(7))
    const list = Node { value: 10, next: Node { value: 20, next: Node { value: 30, next: null } } }
    println(sum_even(list))
    println(sum_odd(list))
    const t = Tree { label: "leaf" }
    println(t.ping(4))
    println(descend_a(0)!)
    match descend_a(3) {
        Ok { value } => { println("ok {value}") }
        Err { error } => { println("err {error}") }
    }
}
