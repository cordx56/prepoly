// Positive examples for the static type-safety rules: exact numeric kinds,
// checked record construction, nullable promotion, and guard-based narrowing.

type SafePoint = {
    x: int32
    y: int32
    label: string
}

fun bump(value: int32?) -> int32 {
    if !value {
        return 0
    }
    return value + 1
}

fun main() {
    let p = SafePoint { x: 20, y: 21, label: "answer" }
    let total: int32 = p.x + p.y
    let maybe_total: int32? = total

    println("{p.label}: {bump(maybe_total)}")

    let absent: int32? = null
    if absent {
        println(absent + 1)
    } else {
        println("absent")
    }
}
