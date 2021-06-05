use std::convert::TryInto;
use std::fmt;
#[derive(PartialEq, Clone, Debug, Eq, PartialOrd)]
pub struct Ident(pub String);
// Trait implementations for Ident
impl Into<Expr> for Ident {
    fn into(self) -> Expr {
        Expr::Ident(self)
    }
}
impl From<&str> for Ident {
    fn from(string: &str) -> Ident {
        Ident(string.to_string())
    }
}
impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Stmt {
    Assign(Pattern, Expr),
    Return(Expr),
    Expr(Expr),
    Import { source: Expr, name: Pattern },
}

impl Into<BlockStatement> for Stmt {
    fn into(self) -> BlockStatement {
        BlockStatement(vec![self])
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Assign(ident, expr) => write!(f, "{} = {}", ident, expr),
            Stmt::Return(expr) => write!(f, "return {}", expr),
            Stmt::Import { source, name } => write!(f, "import {} from {}", name, source),
            Stmt::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Expr {
    Number(f64),
    Ident(Ident),
    Pattern(Pattern),
    Prefix(String, Box<Expr>),
    Infix(Box<Expr>, String, Box<Expr>),
    Boolean(bool),
    String(String),
    Symbol(String),
    If {
        condition: Box<Expr>,
        consequence: BlockStatement,
        alternative: BlockStatement,
    },
    Function {
        parameters: Vec<Ident>,
        body: BlockStatement,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Match {
        condition: Box<Expr>,
        cases: Vec<(Pattern, BlockStatement)>,
    },
    // Array data structure
    Array(Vec<Expr>),
    // Hashmap/dictionary/object data structure
    // It's a Vec because of trait constraints
    Hash(Vec<(Ident, Expr)>),
}

impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expr(self)
    }
}

impl Into<Vec<Stmt>> for Expr {
    fn into(self) -> Vec<Stmt> {
        vec![self.into()]
    }
}
impl Into<BlockStatement> for Expr {
    fn into(self) -> BlockStatement {
        BlockStatement(self.into())
    }
}

// From traits
impl From<&str> for Expr {
    fn from(val: &str) -> Self {
        Expr::String(val.to_string())
    }
}
impl From<Vec<Expr>> for Expr {
    fn from(val: Vec<Expr>) -> Self {
        Expr::Array(val)
    }
}
impl From<bool> for Expr {
    fn from(val: bool) -> Self {
        Expr::Boolean(val)
    }
}

impl From<f64> for Expr {
    fn from(val: f64) -> Self {
        Expr::Number(val)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ident(ident) => write!(f, "{}", ident.0),
            Expr::Pattern(pattern) => write!(f, "{}", pattern),
            Expr::Number(num) => write!(f, "{}", num),
            Expr::Prefix(op, expr) => write!(f, "({}{})", op, expr),
            Expr::Infix(left, operator, right) => write!(f, "({} {} {})", left, operator, right),
            Expr::Boolean(value) => write!(f, "{}", value),
            Expr::String(value) => write!(f, "'{}'", value),
            Expr::Symbol(value) => write!(f, ":{}", value),
            Expr::If {
                condition,
                consequence,
                alternative,
            } => write!(
                f,
                "if {} {{\n{}}} else {{\n{}}}",
                condition, consequence, alternative
            ),
            Expr::Function { parameters, body } => {
                write!(f, "fn (")?;
                let mut params = vec![];
                for param in parameters {
                    params.push(param.0.clone())
                }
                write!(f, "{}", params.join(","))?;
                write!(f, ") -> {{{}}}", body)?;
                Ok(())
            }
            Expr::Call {
                function,
                arguments,
            } => {
                let mut out = String::new();

                out.push_str(&format!("{}", function));
                let mut args = vec![];
                for arg in arguments {
                    args.push(format!("{}", arg))
                }

                out.push('(');
                out.push_str(&args.join(", "));
                out.push(')');

                write!(f, "{}", out)
            }
            Expr::Match { condition, cases } => {
                let mut out = String::new();
                out.push_str(&format!("{} :: {{\n", condition));
                let mut formatted_cases = vec![];
                for (key, value) in cases {
                    formatted_cases.push(format!("{} -> {{\n{}}},", key, value))
                }

                out.push_str(&formatted_cases.join("\n"));
                out.push_str("\n}");

                write!(f, "{}", out)
            }
            Expr::Array(items) => {
                let mut x = vec![];
                for item in items {
                    x.push(format!("{}", item))
                }

                write!(f, "[{}]", x.join(","))
            }
            Expr::Hash(items) => {
                let mut x = vec![];
                for (key, value) in items {
                    x.push(format!("{} = {}", key, value))
                }

                write!(f, "{{{}}}", x.join(", "))
            }
        }
    }
}

impl Default for BlockStatement {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct BlockStatement(pub Vec<Stmt>);
impl BlockStatement {
    pub fn new() -> BlockStatement {
        BlockStatement(vec![])
    }
    pub fn from(stmts: Vec<Stmt>) -> BlockStatement {
        BlockStatement(stmts)
    }
    pub fn expr(expr: Expr) -> BlockStatement {
        BlockStatement(vec![Stmt::Expr(expr)])
    }
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.0.clone();
        for stmt in inner {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}
pub type Program = BlockStatement;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Pattern {
    String(String),
    Number(f64),
    Boolean(bool),
    Symbol(String),
    Ident(Ident),
    // Array destructuring
    Array(Vec<Pattern>),
    // Hashmap destructing
    Hash(Vec<(Ident, Option<Ident>)>),
    Nothing,
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Ident(id) => id.fmt(f),
            Pattern::Number(num) => num.fmt(f),
            Pattern::Boolean(b) => b.fmt(f),
            Pattern::Symbol(sym) => write!(f, ":{}", sym),
            Pattern::String(str) => write!(f, "'{}'", str),
            Pattern::Nothing => write!(f, "_"),
            Pattern::Array(items) => {
                let x: Vec<String> = items.iter().map(|item| format!("{}", item)).collect();
                write!(f, "[ {} ]", x.join(","))
            }
            Pattern::Hash(items) => {
                let x: Vec<String> = items
                    .iter()
                    .map(|(key, alias)| match alias {
                        Some(alias) => format!("{}: {}", key, alias),
                        None => format!("{}", key),
                    })
                    .collect();
                write!(f, "{{ {} }}", x.join(","))
            }
        }
    }
}

impl From<Ident> for Pattern {
    fn from(ident: Ident) -> Pattern {
        Pattern::Ident(ident)
    }
}
