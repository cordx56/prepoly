// Pattern matching: exhaustive sum-type match, literal patterns, wildcard,
// `if let` for a single variant, and fixed-length array destructuring.

type Shape =
    | Circle { radius: float64 }
    | Rectangle { width: float64, height: float64 }
    | Point

fun describe(s: Shape) -> string {
    return match s {
        Circle { radius } => "circle r={radius}",
        Rectangle { width, height } => "rect {width}x{height}",
        Point => "a point",
    }
}

fun area(s: Shape) -> float64 {
    return match s {
        Circle { radius } => 3.14159 * radius * radius,
        Rectangle { width, height } => width * height,
        Point => 0.0,
    }
}

// `if let` matches a single variant and binds its fields.
fun radius_of(s: Shape) -> float64? {
    if let Circle { radius } = s {
        return radius
    }
    return null
}

fun classify(n: int32) -> string {
    return match n {
        0 => "zero",
        1 => "one",
        _ => "many",
    }
}

fun main() {
    let shapes = [
        Shape.Circle { radius: 2.0 },
        Shape.Rectangle { width: 3.0, height: 4.0 },
        Shape.Point,
    ]
    for s in shapes {
        println("{describe(s)} -> area {area(s)}")
    }

    let r = radius_of(Shape.Circle { radius: 5.0 })
    if r {
        println("found radius {r}")
    }

    for n in [0, 1, 2, 9] {
        println("{n} is {classify(n)}")
    }
}
