// Pins comparison-operand typing when an int literal is wider than the
// variable's kind: `b < 300` must compare at the common (wider) type, not
// truncate 300 into uint8 (which made 200 < 44 false). A literal that fits
// still adapts to the variable's kind (`b < 250` compares as uint8).
fun main() {
    let b: uint8 = 200
    if b < 300 {
        println("less")
    } else {
        println("not less")
    }
    if b < 250 {
        println("fits-less")
    } else {
        println("fits-not-less")
    }
    if b > 100 {
        println("greater")
    }
}
