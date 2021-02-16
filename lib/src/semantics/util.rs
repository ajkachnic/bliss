use crate::ast::Expr;

pub fn is_ident(expr: &Expr) -> bool {
    match expr {
        &Expr::Ident(_) => true,
        _ => false,
    }
}

pub fn is_array(expr: &Expr) -> bool {
    match expr {
        &Expr::Array(_) => true,
        _ => false,
    }
}

pub fn is_hash(expr: &Expr) -> bool {
    match expr {
        &Expr::Hash(_) => true,
        _ => false,
    }
}

pub fn is_string(expr: &Expr) -> bool {
    match expr {
        &Expr::String(_) => true,
        _ => false,
    }
}

pub fn is_function(expr: &Expr) -> bool {
    match expr {
        &Expr::Function {
            parameters: _,
            body: _,
        } => true,
        _ => false,
    }
}
