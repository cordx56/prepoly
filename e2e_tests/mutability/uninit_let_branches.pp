type Point = { x: int64, y: int64 }
fun pick(flag: bool) -> Point {
    let p: Point
    if flag {
        p.x = 1
        p.y = 2
    } else {
        p = Point { x: 10, y: 20 }
    }
    return p
}
fun early(flag: bool) -> int64 {
    let n: int64
    if flag {
        return 0
    } else {
        n = 5
    }
    return n
}
fun main() {
    println(pick(true).x + pick(false).y)
    println(early(false))
    let q: Point
    q.x = 7
    println(q.x)
    q.y = 8
    println(q)
}
