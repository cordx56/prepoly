// Printing a record renders the multi-line form documented in types.md,
// including a nested sum variant with its type-qualified name.
type Program =
    | Master { year: int32 }

type Student = {
    first_name: string
    last_name: string
    id
    program: Program
}

fun main() {
    const newton = Student {
        first_name: "Isac",
        last_name: "Newton",
        id: 1001,
        program: Program.Master { year: 1 },
    }
    println("{newton}")
}
