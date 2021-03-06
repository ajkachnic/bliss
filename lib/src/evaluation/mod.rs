pub mod builtins;
pub mod env;
pub mod object;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BlockStatement, Expr, Ident, Pattern, Program, Stmt};
use env::Environment;
use object::Object;

type EvalResult<'a> = Result<Object<'a>, String>;
#[derive(Debug, Clone)]
pub struct Evaluator<'a> {
    env: Rc<RefCell<Environment<'a>>>,
}
impl<'a> Evaluator<'a> {
    pub fn new(env: Rc<RefCell<Environment<'a>>>) -> Self {
        let builtins = builtins::get_builtins();
        for (name, value) in builtins {
            let mut inner_env = env.borrow_mut();
            inner_env.set(name, value);
        }
        Self { env }
    }
    pub fn eval_program(&mut self, program: Program) -> EvalResult<'a> {
        let mut result = Object::Void;
        for stmt in program.0 {
            match self.eval_stmt(stmt)? {
                Object::Return(value) => return Ok(*value),
                value => result = value,
            }
        }
        Ok(result)
    }

    fn eval_block_stmt(&mut self, stmts: BlockStatement) -> EvalResult<'a> {
        let mut result = Object::Void;
        for stmt in stmts.0 {
            match self.eval_stmt(stmt)? {
                Object::Return(value) => return Ok(Object::Return(value)),
                value => result = value,
            }
        }
        Ok(result)
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> EvalResult<'a> {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(Object::Return(Box::new(value)))
            }
            Stmt::Assign(name, value) => {
                let value = self.eval_expr(value)?;
                match name {
                    Pattern::Ident(ident) => {
                        self.env.borrow_mut().set(ident.0, value);
                    }
                    Pattern::Array(names) => {
                        if let Object::Array(values) = value {
                            names.iter().enumerate().for_each(|(index, name)| {
                                if let Pattern::Ident(name) = name {
                                    self.env
                                        .borrow_mut()
                                        .set(name.0.clone(), values[index].clone())
                                }
                            });
                        }
                    }
                    _ => {}
                }
                Ok(Object::Void)
            }
            _ => Err("statement not supported".to_string()),
        }
    }

    fn eval_expr(&mut self, node: Expr) -> EvalResult<'a> {
        match node {
            Expr::Number(value) => Ok(Object::Number(value)),
            Expr::String(value) => Ok(Object::String(value)),
            Expr::Boolean(value) => Ok(Self::native_bool_to_object(value)),
            Expr::Array(value) => {
                let mut items = vec![];
                for part in value {
                    let item = self.eval_expr(part)?;
                    items.push(item)
                }
                Ok(Object::Array(items))
            }
            Expr::Hash(values) => {
                let mut hash = HashMap::new();
                for (key, value) in values {
                    let value = self.eval_expr(value)?;
                    hash.insert(key.0, value);
                }

                Ok(Object::Hash(hash))
            }
            Expr::Prefix(operator, right) => {
                let right = self.eval_expr(*right)?;
                self.eval_prefix_expression(&operator, right)
            }
            Expr::Infix(left, operator, right) => {
                let left = self.eval_expr(*left)?;
                let right = self.eval_expr(*right)?;
                self.eval_infix_expression(left, &operator, right)
            }
            Expr::If {
                condition,
                consequence,
                alternative,
            } => self.eval_if_expression(*condition, consequence, alternative),
            Expr::Match { condition, cases } => self.eval_match_expression(*condition, cases),
            Expr::Ident(name) => match self.env.borrow().get(name.0.clone()) {
                Some(value) => Ok(value),
                None => {
                    // println!("{:#?}", self.env);
                    Err(format!("Identifier not found: {}", name))
                }
            },
            Expr::Call {
                function,
                arguments,
            } => self.eval_call_expression(*function, arguments),
            Expr::Member {
                property,
                object,
                computed,
            } => self.eval_member_expression(*property, *object, computed),
            Expr::Function { parameters, body } => Ok(Object::Function {
                parameters,
                body,
                env: Environment::new_enclosed(&self.env).into(),
            }),
            Expr::Symbol(sym) => Ok(Object::Symbol(sym)),
        }
    }

    fn eval_function_call(
        &mut self,
        function: Object<'a>,
        args: Vec<Object<'a>>,
    ) -> EvalResult<'a> {
        let (params, body, env) = match function {
            Object::Function {
                parameters,
                body,
                env,
            } => (parameters, body, env),
            Object::Builtin(params, func) => {
                if params < 0 || params == (args.len() as isize) {
                    return func(args, Rc::new(RefCell::new(self.clone())));
                }
                return Err(format!(
                    "Incorrect number of arguments passed: expected {}, got {}",
                    params,
                    args.len()
                ));
            }
            _ => return Err(format!("Cannot call expression {}", function)),
        };
        if params.len() != args.len() {
            return Err(format!(
                "Wrong number of arguments: expected {}, found {}",
                params.len(),
                args.len()
            ));
        }

        let mut function_env = Environment::new_enclosed(&Rc::clone(&env));
        params
            .iter()
            .enumerate()
            .for_each(|(index, param)| function_env.set(param.0.clone(), args[index].clone()));
        self.env = Rc::new(RefCell::new(function_env));
        let res = self.eval_block_stmt(body)?;

        self.env = env;
        Ok(res)
    }

    fn eval_call_expression(&mut self, function: Expr, arguments: Vec<Expr>) -> EvalResult<'a> {
        let function = self.eval_expr(function)?;
        let mut args = vec![];
        for arg in arguments {
            let res = self.eval_expr(arg)?;
            args.push(res);
        }

        let (params, body, env) = match function {
            Object::Function {
                parameters,
                body,
                env,
            } => (parameters, body, env),
            Object::Builtin(params, func) => {
                if params < 0 || params == (args.len() as isize) {
                    return func(args, Rc::new(RefCell::new(self.clone())));
                }
                return Err(format!(
                    "Incorrect number of arguments passed: expected {}, got {}",
                    params,
                    args.len()
                ));
            }
            _ => return Err(format!("Cannot call expression {}", function)),
        };
        if params.len() != args.len() {
            return Err(format!(
                "Wrong number of arguments: expected {}, found {}",
                params.len(),
                args.len()
            ));
        }

        let current_env = Rc::clone(&env);
        let mut function_env = Environment::new_enclosed(&current_env);
        params
            .iter()
            .enumerate()
            .for_each(|(index, param)| function_env.set(param.0.clone(), args[index].clone()));
        self.env = Rc::new(RefCell::new(function_env));
        let res = self.eval_block_stmt(body)?;

        self.env = env;
        Ok(res)
    }

    fn eval_member_expression(
        &mut self,
        property: Expr,
        object: Expr,
        computed: bool,
    ) -> EvalResult<'a> {
        let object = self.eval_expr(object)?;
        let property = if computed {
            self.eval_expr(property)?
        } else {
            match property {
                Expr::Ident(ident) => Object::Ident(ident),
                _ => unreachable!(),
            }
        };

        self.eval_member_components(property, object, computed)
    }
    fn eval_member_components(
        &mut self,
        property: Object<'a>,
        object: Object<'a>,
        computed: bool,
    ) -> EvalResult<'a> {
        Ok(match (property, object) {
            (Object::Number(n), Object::Array(arr)) => arr.get(n as usize).unwrap().clone(),
            (Object::String(str), Object::Hash(hash)) => hash.get(&str).unwrap().clone(),
            (Object::Ident(property), object) => {
                let index = match computed {
                    true => self.env.borrow().get(property.to_string()).unwrap(),
                    false => Object::String(property.to_string()),
                };
                return self.eval_member_components(index, object, computed);
            }
            (index, of) => return Err(format!("Incompatible types, {} and {}", index, of)),
        })
    }

    fn eval_match_expression(
        &mut self,
        condition: Expr,
        cases: Vec<(Pattern, BlockStatement)>,
    ) -> EvalResult<'a> {
        let condition = self.eval_expr(condition)?;
        let current = Rc::clone(&self.env);
        let mut result = Object::Void;

        for (case, consequence) in cases {
            let env = Rc::clone(&current);
            let evaled = self.eval_pattern_matching(&env, case, condition.clone())?;
            if self.eval_match_case(evaled, condition.clone()) {
                self.env = env;
                result = self.eval_block_stmt(consequence)?;
                self.env = current;
                break;
            }
        }
        Ok(result)
    }

    fn eval_match_case(&mut self, case: Object<'a>, condition: Object<'a>) -> bool {
        match case {
            Object::Array(array) => {
                if let Object::Array(condition) = condition {
                    for (index, element) in array.iter().enumerate() {
                        let result =
                            self.eval_match_case(element.clone(), condition[index].clone());
                        if !result {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            Object::Ident(_) => true,
            _ => case == condition,
        }
    }

    fn eval_pattern_matching(
        &mut self,
        env: &Rc<RefCell<Environment<'a>>>,
        case: Pattern,
        condition: Object<'a>,
    ) -> EvalResult<'a> {
        let value = match case {
            Pattern::Nothing => Object::Ident(Ident::from("_")),
            Pattern::Ident(ident) => {
                env.borrow_mut().set(ident.0, condition.clone());
                condition
            }
            Pattern::Array(array) => {
                let mut arr = vec![];
                let condition_array = match condition {
                    Object::Array(a) => a,
                    _ => return Err("Mismatched types".to_string()),
                };
                for (pattern, condition) in array.iter().zip(condition_array) {
                    let result =
                        self.eval_pattern_matching(env, pattern.clone(), condition.clone())?;
                    arr.push(result);
                }
                Object::Array(arr)
            }
            Pattern::Hash(hash) => {
                let mut arr = vec![];
                let condition_hash = match condition {
                    Object::Hash(a) => a,
                    _ => return Err("Mismatched types".to_string()),
                };
                for (key, alias) in hash {
                    let condition = match condition_hash.get(&key.0) {
                        Some(cond) => cond,
                        None => &Object::Null,
                    };
                    let pattern = Pattern::Ident(alias.unwrap_or(key));
                    let result = self.eval_pattern_matching(env, pattern, condition.clone())?;
                    arr.push(result);
                }
                Object::Array(arr)
            }
            Pattern::String(str) => Object::String(str),
            Pattern::Number(num) => Object::Number(num),
            Pattern::Symbol(str) => Object::Symbol(str),
            Pattern::Boolean(bool) => Object::Boolean(bool),
        };

        Ok(value)
    }

    fn eval_infix_expression(
        &self,
        left: Object<'a>,
        operator: &str,
        right: Object<'a>,
    ) -> EvalResult<'a> {
        match operator {
            "+" => self.eval_plus_operator(left, right),
            "-" | "*" | "/" | "%" | ">" | "<" | ">=" | "<=" => {
                self.eval_number_operator(left, operator, right)
            }
            "==" | "!=" => self.eval_boolean_operator(left, operator, right),
            ".." => self.eval_range_operator(left, right),
            _ => Err("Unsupported operator".to_string()),
        }
    }

    fn eval_prefix_expression(&self, operator: &str, right: Object) -> EvalResult<'a> {
        match operator {
            "!" => Ok(self.eval_bang_operator(right)),
            "-" => {
                if let Object::Number(right) = right {
                    return Ok(Object::Number(-right));
                }
                Err(format!("Can't negate {}", right))
            }
            _ => Err(format!("Couldn't evaluate operator {}", operator)),
        }
    }

    fn eval_number_operator(&self, left: Object, operator: &str, right: Object) -> EvalResult<'a> {
        let result = if let Object::Number(left) = left {
            if let Object::Number(right) = right {
                match operator {
                    "-" => Ok(Object::Number(left - right)),
                    "*" => Ok(Object::Number(left * right)),
                    "/" => Ok(Object::Number(left / right)),
                    "%" => Ok(Object::Number(left % right)),

                    ">" => Ok(Object::from(left > right)),
                    "<" => Ok(Object::from(left < right)),
                    ">=" => Ok(Object::from(left >= right)),
                    "<=" => Ok(Object::from(left <= right)),
                    _ => Err(format!("invalid operator {}", operator)),
                }
            } else {
                Err(format!(
                    "Can't use {} on {:?} and {:?}",
                    operator, left, right
                ))
            }
        } else {
            Err(format!(
                "Can't use {} on {:?} and {:?}",
                operator, left, right
            ))
        }?;

        Ok(result)
    }

    fn eval_boolean_operator(
        &self,
        left: Object<'a>,
        operator: &str,
        right: Object<'a>,
    ) -> EvalResult<'a> {
        match operator {
            "==" => Ok(Self::native_bool_to_object(left == right)),
            "!=" => Ok(Self::native_bool_to_object(left != right)),
            _ => Err(format!("Invalid operator {}", operator)),
        }
    }

    fn eval_range_operator(&self, left: Object<'a>, right: Object<'a>) -> EvalResult<'a> {
        if let Object::Number(left) = left {
            if let Object::Number(right) = right {
                let left = left.round() as i64;
                let right = right.round() as i64;
                let mut items = vec![];
                for item in left..right {
                    items.push(Object::Number(item as f64));
                }

                return Ok(Object::Array(items));
            }
        }
        Err(format!(
            "Can't use range operator on {} and {}",
            left, right
        ))
    }

    fn eval_plus_operator(&self, left: Object<'a>, right: Object<'a>) -> EvalResult<'a> {
        match left {
            Object::Number(left) => {
                if let Object::Number(right) = right {
                    return Ok(Object::Number(left + right));
                }
                Err(format!("Unable to add {:?} and {:?}", left, right))
            }
            Object::String(left) => {
                if let Object::String(right) = right {
                    let new = [left, right].concat();
                    return Ok(Object::String(new));
                }
                Err(format!("Unable to add {:?} and {:?}", left, right))
            }
            Object::Array(left) => {
                if let Object::Array(right) = right {
                    let new = [left, right].concat();
                    return Ok(Object::Array(new));
                }
                Err(format!("Unable to add {:?} and {:?}", left, right))
            }
            _ => Err(format!("Unable to add {:?} and {:?}", left, right)),
        }
    }

    fn eval_bang_operator(&self, right: Object) -> Object<'a> {
        Self::native_bool_to_object(!Self::is_truthy(right))
    }

    fn eval_if_expression(
        &mut self,
        condition: Expr,
        consequence: BlockStatement,
        alternative: BlockStatement,
    ) -> EvalResult<'a> {
        let condition = self.eval_expr(condition)?;
        if Self::is_truthy(condition) {
            self.eval_block_stmt(consequence)
        } else {
            self.eval_block_stmt(alternative)
        }
    }

    fn native_bool_to_object(input: bool) -> Object<'a> {
        if input {
            Object::Boolean(true)
        } else {
            Object::Boolean(false)
        }
    }

    fn is_truthy(input: Object) -> bool {
        matches!(input, Object::Boolean(true))
    }
}
