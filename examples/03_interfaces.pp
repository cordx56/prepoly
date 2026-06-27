// Interface enforcement: `type User: Showable, Comparable` requires User to
// provide every member of each interface (checked at compile time). No
// implementation is inherited. Structural subtyping then lets a function that
// only needs `to_string` accept any type that has it.

type Showable = {
    to_string(self) -> string
}

type Comparable = {
    compare(self, other) -> int32
}

type User: Showable, Comparable = {
    name: string
    age: int32

    new(name: string, age: int32) {
        return Self { name: name, age: age }
    }

    to_string(self) -> string {
        return "{self.name} (age {self.age})"
    }

    compare(self, other) -> int32 {
        return self.age - other.age
    }
}

// Accepts anything with a `to_string` method (structural subtyping).
fun print_info(obj) {
    println(obj.to_string())
}

fun main() {
    let a = User.new("Alice", 30)
    let b = User.new("Bob", 25)
    print_info(a)
    print_info(b)
    println("a.compare(b) = {a.compare(b)}")
}
