// A module: this file is `geometry.vec`. Public names (no leading `_`) are
// importable; `_`-prefixed names stay private to the file.

type Vec2 = {
    x: float64
    y: float64

    new(x: float64, y: float64) {
        return Self { x: x, y: y }
    }

    add(self, other: Vec2) -> Vec2 {
        return Self { x: self.x + other.x, y: self.y + other.y }
    }

    scale(self, k: float64) -> Vec2 {
        return Self { x: self.x * k, y: self.y * k }
    }

    length(self) -> float64 {
        return sqrt(self.x * self.x + self.y * self.y)
    }
}

fun dot(a: Vec2, b: Vec2) -> float64 {
    return a.x * b.x + a.y * b.y
}
