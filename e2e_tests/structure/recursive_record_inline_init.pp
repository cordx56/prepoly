// A non-null value initializing a declared-nullable field inline (building a
// linked list front-to-back in a loop, inside a function) keeps the field's
// declared nullable type: the value is wrapped into the nullable cell at the
// store, and destructors/readers see the cell layout they expect.
type Node = { value: int64, next: Node? }

fun build(n: int64) -> Node {
    let head = Node { value: 0, next: null }
    let i: int64 = 1
    while i < n {
        head = Node { value: i, next: head }
        i += 1
    }
    return head
}

fun sum(head: Node) -> int64 {
    let total: int64 = head.value
    let nxt = head.next
    if nxt {
        total += sum(nxt)
    }
    return total
}

fun main() {
    let list = build(5)
    println(list.value)
    println(sum(list))
}
