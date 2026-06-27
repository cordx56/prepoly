// Interface enforcement on a sum type: every variant of `Pet` must satisfy
// `Named` (have a `name: string` field). Shared fields are still read out by
// pattern matching, since variants are nominal.

type Named = {
    name: string
}

type Pet: Named =
    | Cat { name: string, indoor: bool }
    | Dog { name: string, breed: string }

fun greet(p: Pet) {
    match p {
        Cat { name, indoor } => {
            let status = if indoor { "indoor" } else { "outdoor" }
            println("{name} is an {status} cat")
        },
        Dog { name, breed } => {
            println("{name} is a {breed} dog")
        },
    }
}

fun main() {
    greet(Pet.Cat { name: "Tama", indoor: true })
    greet(Pet.Dog { name: "Pochi", breed: "Shiba" })
}
