//! Parser for the Prepoly language: tokens -> AST.

pub mod ast;
pub mod newline;
mod parser;

pub use parser::{ParseError, parse, parse_with_base};

#[cfg(test)]
mod tests {
    use super::ast::*;
    use super::parse;

    fn module(src: &str) -> Module {
        parse(src).unwrap_or_else(|e| panic!("parse error: {} at {:?}", e.message, e.span))
    }

    #[test]
    fn record_type_with_methods() {
        let m = module(
            "type Point = {\n    x: float64\n    y: float64\n    new(x: float64, y: float64) {\n        return Self { x: x, y: y }\n    }\n    dist(self, o: Point) -> float64 {\n        return self.x\n    }\n}\n",
        );
        match &m.items[0] {
            TopLevel::Type(t) => {
                assert_eq!(t.name, "Point");
                match &t.body {
                    TypeBody::Record(members) => {
                        assert_eq!(members.len(), 4);
                        assert!(matches!(members[0], Member::Field(_)));
                        assert!(matches!(members[2], Member::Method(_)));
                    }
                    _ => panic!("expected record"),
                }
            }
            _ => panic!("expected type decl"),
        }
    }

    #[test]
    fn sum_type_with_leading_pipe() {
        let m = module("type Color =\n    | Red\n    | Green\n    | Blue\n");
        match &m.items[0] {
            TopLevel::Type(t) => match &t.body {
                TypeBody::Sum(vs) => {
                    let names: Vec<_> = vs.iter().map(|v| v.name.as_str()).collect();
                    assert_eq!(names, vec!["Red", "Green", "Blue"]);
                }
                _ => panic!("expected sum"),
            },
            _ => panic!("expected type"),
        }
    }

    #[test]
    fn interface_constraints() {
        let m = module("type User: Showable, Comparable = {\n    name: string\n}\n");
        match &m.items[0] {
            TopLevel::Type(t) => assert_eq!(t.interfaces, vec!["Showable", "Comparable"]),
            _ => panic!(),
        }
    }

    #[test]
    fn method_chain_newline_continuation() {
        let m = module(
            "fun main() {\n    let r = items\n        .filter((x) -> x > 0)\n        .map((x) -> x * 2)\n}\n",
        );
        // Should parse without treating the newline before `.` as a terminator.
        match &m.items[0] {
            TopLevel::Fun(f) => assert_eq!(f.body.stmts.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn trailing_operator_continuation() {
        let m = module("fun main() {\n    let total = price *\n        quantity\n}\n");
        match &m.items[0] {
            TopLevel::Fun(f) => {
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0] {
                    Stmt::Let { value, .. } => {
                        assert!(matches!(value, Expr::Binary(BinOp::Mul, ..)))
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn match_with_patterns() {
        let m = module(
            "fun f(s) {\n    return match s {\n        Circle { radius } => radius,\n        Point => 0,\n        _ => 1,\n    }\n}\n",
        );
        match &m.items[0] {
            TopLevel::Fun(f) => match &f.body.stmts[0] {
                Stmt::Return(Some(Expr::Match(_, arms, _)), _) => assert_eq!(arms.len(), 3),
                _ => panic!("expected match return"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn variant_literal_and_field_access() {
        let m = module(
            "fun main() {\n    let c = Animal.Cat { name: \"Tama\", indoor: true }\n    let r = Color.Red\n}\n",
        );
        match &m.items[0] {
            TopLevel::Fun(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let {
                        value: Expr::VariantLit(t, v, fields, _),
                        ..
                    } => {
                        assert_eq!(t, "Animal");
                        assert_eq!(v, "Cat");
                        assert_eq!(fields.len(), 2);
                    }
                    _ => panic!("expected variant literal"),
                }
                match &f.body.stmts[1] {
                    Stmt::Let {
                        value: Expr::Field(_, name, _),
                        ..
                    } => assert_eq!(name, "Red"),
                    _ => panic!("expected field access for unit variant"),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn array_destructuring_let() {
        let m = module("fun main() {\n    let [lo, hi] = [1, 10]\n}\n");
        match &m.items[0] {
            TopLevel::Fun(f) => match &f.body.stmts[0] {
                Stmt::Let {
                    pat: Pattern::Array(ps, _),
                    ..
                } => assert_eq!(ps.len(), 2),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn string_interpolation_expr() {
        let m = module("fun main() {\n    let s = \"value = {x + 1}\"\n}\n");
        match &m.items[0] {
            TopLevel::Fun(f) => match &f.body.stmts[0] {
                Stmt::Let {
                    value: Expr::Str(segs, _),
                    ..
                } => {
                    assert_eq!(segs.len(), 2);
                    assert!(matches!(segs[0], StrSeg::Lit(_)));
                    assert!(matches!(segs[1], StrSeg::Expr(_)));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn imports() {
        let m = module("import math.vector.{ Vec2, Vec3 }\nimport config.{ Config }\n");
        assert_eq!(m.imports.len(), 2);
        assert_eq!(m.imports[0].path, vec!["math", "vector"]);
        assert_eq!(m.imports[0].names, vec!["Vec2", "Vec3"]);
        assert_eq!(m.imports[1].path, vec!["config"]);
    }

    #[test]
    fn if_let_and_else() {
        let m = module(
            "fun f(s) {\n    if let Circle { radius } = s {\n        return radius\n    } else {\n        return 0\n    }\n}\n",
        );
        match &m.items[0] {
            TopLevel::Fun(f) => match &f.body.stmts[0] {
                Stmt::Expr(Expr::IfLet(_, _, _, els, _)) => assert!(els.is_some()),
                _ => panic!("expected if let"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn closure_vs_grouping() {
        let m = module(
            "fun main() {\n    let a = (1 + 2)\n    let f = (x) -> x * 2\n    let g = () -> { return 1 }\n}\n",
        );
        match &m.items[0] {
            TopLevel::Fun(f) => {
                assert!(matches!(
                    f.body.stmts[0],
                    Stmt::Let {
                        value: Expr::Binary(..),
                        ..
                    }
                ));
                assert!(matches!(
                    f.body.stmts[1],
                    Stmt::Let {
                        value: Expr::Closure(..),
                        ..
                    }
                ));
                assert!(matches!(
                    f.body.stmts[2],
                    Stmt::Let {
                        value: Expr::Closure(..),
                        ..
                    }
                ));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn star_star_is_not_an_operator() {
        // `**` was removed as an operator; exponentiation is the `pow` function in
        // std/math. `2 ** 3` no longer lexes as a single token, and `*` is not a
        // prefix operator, so the expression must fail to parse.
        assert!(parse("fun main() {\n    let x = 2 ** 3\n}\n").is_err());
    }

    #[test]
    fn nullable_and_array_types() {
        let m = module("fun f(a: int32?, b: string[], c: float64[3]) {\n}\n");
        match &m.items[0] {
            TopLevel::Fun(f) => {
                assert!(matches!(f.params[0].ty, Some(TypeExpr::Nullable(..))));
                assert!(matches!(f.params[1].ty, Some(TypeExpr::Array(_, None, _))));
                assert!(matches!(
                    f.params[2].ty,
                    Some(TypeExpr::Array(_, Some(3), _))
                ));
            }
            _ => panic!(),
        }
    }
}
