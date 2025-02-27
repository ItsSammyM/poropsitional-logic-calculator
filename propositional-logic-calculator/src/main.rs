use std::collections::HashMap;

fn get_user_input()->String{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    return line
}


fn main() {
    println!("Hello, world!");
    let input = get_user_input();
    println!("{:?}", ExpressionParser::parse_string(&input));
    
}

#[derive(Debug, Clone)]
pub struct Variable(u8);
impl Variable{
    fn new(name: u8)->Self{Self(name)}
}

#[derive(Debug)]
pub enum Expression{
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Variable(Variable),
    True,
    False
}

impl Expression{
    pub fn new_true()->Box<Self>{Box::new(Self::True)}
    pub fn new_false()->Box<Self>{Box::new(Self::False)}
    pub fn new_variable(var: Variable)->Box<Self>{Box::new(Self::Variable(var))}
    pub fn new_and(a: Box<Expression>, b: Box<Expression>)->Box<Self>{Box::new(Self::And(a, b))}
    pub fn new_or(a: Box<Expression>, b: Box<Expression>)->Box<Self>{Box::new(Self::Or(a, b))}
    pub fn new_not(a: Box<Expression>)->Box<Self>{Box::new(Self::Not(a))}
}


/// For parsing
/// Key characters are "!|& _()"
/// "_" and " " are ignored
/// 
/// CFGs
/// Prototype
/// Expr -> "AND" | "OR" | "NOT" | "TRUE" | "FALSE" | "VAR" | "(EXPR)"
/// AND -> "EXPR & EXPR"
/// OR -> "EXPR | EXPR"
/// NOT -> "!EXPR"
/// 
/// Used one
/// Expr -> Expr "|" AndExpr | AndExpr
/// AndExpr -> AndExpr "&" NotExpr | NotExpr
/// NotExpr -> "!" NotExpr | Primary
/// Primary -> "(" Expr ")" | Identifier | "true" | "false"
#[derive(Default)]
struct ExpressionParser {
    variable_names: HashMap<String, Variable>,
    next_var: u8,
    chars: Vec<char>,
    pos: usize,
}

impl ExpressionParser {
    pub fn parse_string(string: &str) -> Box<Expression> {
        let filtered: String = string
            .chars()
            .filter(|c| !['_', ' ', '\n'].contains(c))
            .collect();
        let chars = filtered.chars().collect();
        let mut parser = ExpressionParser {
            variable_names: HashMap::new(),
            next_var: 0,
            chars,
            pos: 0,
        };
        parser.parse_expression()
    }

    fn parse_expression(&mut self) -> Box<Expression> {
        let mut expr = self.parse_and();
        while self.consume_if('|') {
            let rhs = self.parse_and();
            expr = Expression::new_or(expr, rhs);
        }
        expr
    }

    fn parse_and(&mut self) -> Box<Expression> {
        let mut expr = self.parse_not();
        while self.consume_if('&') {
            let rhs = self.parse_not();
            expr = Expression::new_and(expr, rhs);
        }
        expr
    }

    fn parse_not(&mut self) -> Box<Expression> {
        if self.consume_if('!') {
            let expr = self.parse_not();
            Expression::new_not(expr)
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Box<Expression> {
        if self.consume_if('(') {
            let expr = self.parse_expression();
            self.expect(')');
            expr
        } else if self.consume_str("true") {
            Expression::new_true()
        } else if self.consume_str("false") {
            Expression::new_false()
        } else {
            let var_name = self.parse_var_name();
            let var = self.get_or_create_var(var_name);
            Expression::new_variable(var)
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn consume_if(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) {
        if !self.consume_if(c) {
            panic!("Expected '{}'", c);
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if self.pos + s_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &sc) in s_chars.iter().enumerate() {
            if self.chars[self.pos + i] != sc {
                return false;
            }
        }
        self.pos += s_chars.len();
        true
    }

    fn parse_var_name(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn get_or_create_var(&mut self, name: String) -> Variable {
        if let Some(var) = self.variable_names.get(&name) {
            var.clone()
        } else {
            let var = Variable::new(self.next_var);
            self.next_var += 1;
            self.variable_names.insert(name, var.clone());
            var
        }
    }
}