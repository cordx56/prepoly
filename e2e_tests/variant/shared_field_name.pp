type J =
    | Number { value: int32 }
    | Text { value: string }

fun show(j: J) -> string {
    return match j {
        Number { value } => "num",
        Text { value } => value,
    }
}

fun main() {
    println(show(J.Text { value: "hi" }))
    println(show(J.Number { value: 42 }))
}
