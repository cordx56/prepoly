// Methods are implemented with `fun T.m(self, ...)` outside the type body, in the
// same module that declares the type. A method is in scope wherever the type is,
// with no separate import, and its inferred return type flows to the caller.

type Vec2 = {
    x: float64
    y: float64
}

fun Vec2.length_sq(self) -> float64 {
    return self.x * self.x + self.y * self.y
}

fun Vec2.scaled(self, k: float64) {
    return Vec2 { x: self.x * k, y: self.y * k }
}

fun main() {
    let v = Vec2 { x: 3.0, y: 4.0 }

    let sq: float64 = v.length_sq()
    println("length_sq = {sq}")

    let big = v.scaled(2.0)
    println("scaled = ({big.x}, {big.y})")
}
