use crate::ast::Expr;

pub fn is_ident(expr: &Expr) -> bool {
    matches!(expr, Expr::Ident(_))
}


pub fn is_string(expr: &Expr) -> bool {
    matches!(expr, Expr::String(_))
}
