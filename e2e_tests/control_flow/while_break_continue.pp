// `break` and `continue` in `while` and `for` loops.
fun main() {
    let i = 0
    let sum = 0
    while true {
        i += 1
        if i > 10 {
            break
        }
        if i % 2 == 1 {
            continue
        }
        sum += i
    }
    println(sum)

    let found = -1
    for x in [3, 7, 11, 15] {
        if x > 10 {
            found = x
            break
        }
    }
    println(found)
}
