// Sum types (tagged unions): variants with fields and per-variant methods,
// nested pattern matching, and recursive data (an expression tree).

type Color =
    | Red
    | Green
    | Blue

type Expr =
    | Num { value: int32 }
    | BinOp { op: string, left: Expr, right: Expr }
    | Neg { inner: Expr }

fun eval(e: Expr) -> int32 {
    return match e {
        Num { value } => value,
        BinOp { op, left, right } => {
            let l = eval(left)
            let r = eval(right)
            match op {
                "+" => l + r,
                "-" => l - r,
                "*" => l * r,
                _ => 0,
            }
        },
        Neg { inner } => -eval(inner),
    }
}

fun name_of(c: Color) -> string {
    return match c {
        Red => "red",
        Green => "green",
        Blue => "blue",
    }
}

fun main() {
    // 1 + 2 * 3
    let expr = Expr.BinOp {
        op: "+",
        left: Expr.Num { value: 1 },
        right: Expr.BinOp {
            op: "*",
            left: Expr.Num { value: 2 },
            right: Expr.Num { value: 3 },
        },
    }
    println("result = {eval(expr)}")
    println("negate 5 = {eval(Expr.Neg { inner: Expr.Num { value: 5 } })}")
    println("color = {name_of(Color.Green)}")
}
