use std::collections::HashMap;

use super::{expression::ExpressionNode, variable::Variable};


#[derive(Debug, Clone, PartialEq)]
enum ExpressionParserToken {
    Or,
    And,
    Xor,
    Not,
    OpenParenthesis,
    CloseParenthesis,
    ImpliesRight,
    ImpliesLeft,
    Biconditional,
    Variable(String),
}

#[derive(PartialEq, Debug)]
enum Assoc {
    Left,
    Right,
}

impl ExpressionParserToken {
    fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            Self::Or
                | Self::And
                | Self::Xor
                | Self::ImpliesRight
                | Self::ImpliesLeft
                | Self::Biconditional
        )
    }

    fn precedence(&self) -> Result<u8, ExpressionParseError> {
        match self {
            Self::Biconditional => Ok(1),
            Self::ImpliesRight | Self::ImpliesLeft => Ok(2),
            Self::Or => Ok(3),
            Self::Xor => Ok(4),
            Self::And => Ok(5),
            _ => Err(ExpressionParseError::General), //panic!("Not a binary operator"),
        }
    }

    fn associativity(&self) -> Result<Assoc, ExpressionParseError> {
        match self {
            Self::ImpliesRight | Self::ImpliesLeft => Ok(Assoc::Right),
            Self::Biconditional => Ok(Assoc::Left),
            Self::Or | Self::Xor | Self::And => Ok(Assoc::Left),
            _ => Err(ExpressionParseError::General), //panic!("Not a binary operator"),
        }
    }
}

/// For parsing
/// Key characters are "<>^!|& _()"
/// "_" and " " are ignored
pub(super) struct ExpressionParser<'a> {
    variable_names: &'a mut HashMap<String, Variable>,
    tokens: Vec<ExpressionParserToken>,
    current: usize,
}
#[derive(Debug)]
pub enum ExpressionParseError{
    General
}

impl<'a> ExpressionParser<'a> {
    pub fn parse_string(string: &str, variable_names: &mut HashMap<String, Variable>) -> Result<Box<ExpressionNode>, ExpressionParseError> {
        let filtered: String = string
            .chars()
            .filter(|c| !['_', ' ', '\n'].contains(c))
            .collect();
        let tokens = ExpressionParser::tokenize(&filtered);
        let mut parser = ExpressionParser {
            variable_names,
            tokens,
            current: 0,
        };
        let expr = parser.parse_expression(0)?;
        Ok(expr)
    }

    fn tokenize(s: &str) -> Vec<ExpressionParserToken> {
        let chars: Vec<char> = s.chars().collect();
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match c {
                '<' => {
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(ExpressionParserToken::Biconditional);
                        i += 2;
                    } else {
                        tokens.push(ExpressionParserToken::ImpliesLeft);
                        i += 1;
                    }
                }
                '>' => {
                    tokens.push(ExpressionParserToken::ImpliesRight);
                    i += 1;
                }
                '^' => {
                    tokens.push(ExpressionParserToken::Xor);
                    i += 1;
                }
                '!' => {
                    tokens.push(ExpressionParserToken::Not);
                    i += 1;
                }
                '|' => {
                    tokens.push(ExpressionParserToken::Or);
                    i += 1;
                }
                '&' => {
                    tokens.push(ExpressionParserToken::And);
                    i += 1;
                }
                '(' => {
                    tokens.push(ExpressionParserToken::OpenParenthesis);
                    i += 1;
                }
                ')' => {
                    tokens.push(ExpressionParserToken::CloseParenthesis);
                    i += 1;
                }
                _ => {
                    let start = i;
                    while i < chars.len()
                        && !matches!(
                            chars[i],
                            '<' | '>' | '^' | '!' | '|' | '&' | '(' | ')'
                        )
                    {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    if name.is_empty() {
                        panic!("Unexpected character: {}", c);
                    }
                    tokens.push(ExpressionParserToken::Variable(name.to_lowercase()))
                }
            }
        }
        tokens
    }

    fn parse_expression(&mut self, min_precedence: u8) -> Result<Box<ExpressionNode>, ExpressionParseError> {
        let mut left = self.parse_prefix()?;
        loop {
            // Peek the next token and clone it to avoid holding the reference
            let token = match self.peek_token() {
                Some(t) => t.clone(),
                None => break,
            };
            
            // Check if the token is a binary operator with sufficient precedence
            if !token.is_binary_operator() {
                break;
            }
            let precedence = token.precedence()?;
            if precedence < min_precedence {
                break;
            }
            let assoc = token.associativity()?;
            
            // Consume the token after cloning, releasing the immutable borrow
            self.consume_token();
            
            // Determine the next minimum precedence based on associativity
            let next_min = match assoc {
                Assoc::Left => precedence + 1,
                Assoc::Right => precedence,
            };
            
            // Recursively parse the right-hand side
            let right = self.parse_expression(next_min)?;
            
            // Combine the left and right expressions
            left = self.combine_binary(&token, left, right)?;
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Box<ExpressionNode>, ExpressionParseError> {
        let Some(token) = self.consume_token() else {return Err(ExpressionParseError::General)};
        match token {
            ExpressionParserToken::Variable(name) => {
                // Capture the length BEFORE the entry borrow
                let current_len = self.variable_names.len() as u8;
                let var = self
                    .variable_names
                    .entry(name)
                    .or_insert_with(|| Variable::new(current_len)) // Use captured value
                    .clone();
                Ok(ExpressionNode::new_variable(var))
            }
            ExpressionParserToken::OpenParenthesis => {
                let expr = self.parse_expression(0);
                self.expect(ExpressionParserToken::CloseParenthesis)?;
                expr
            }
            ExpressionParserToken::Not => {
                let expr = self.parse_expression(6)?; // Precedence of Not is 6
                Ok(ExpressionNode::new_not(expr))
            }
            _ => Err(ExpressionParseError::General) //panic!("Unexpected token in prefix position: {:?}", token),
        }
    }

    fn combine_binary(
        &self,
        token: &ExpressionParserToken,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    ) -> Result<Box<ExpressionNode>, ExpressionParseError> {
        match token {
            ExpressionParserToken::And => Ok(ExpressionNode::new_and(left, right)),
            ExpressionParserToken::Or => Ok(ExpressionNode::new_or(left, right)),
            ExpressionParserToken::Xor => Ok(ExpressionNode::new_xor(left, right)),
            ExpressionParserToken::ImpliesRight => Ok(ExpressionNode::new_implies(left, right)),
            ExpressionParserToken::ImpliesLeft => Ok(ExpressionNode::new_implies(right, left)),
            ExpressionParserToken::Biconditional => Ok(ExpressionNode::new_biconditional(left, right)),
            _ => Err(ExpressionParseError::General) //panic!("Unexpected binary operator: {:?}", token),
        }
    }

    fn peek_token(&self) -> Option<&ExpressionParserToken> {
        self.tokens.get(self.current)
    }

    fn consume_token(&mut self) -> Option<ExpressionParserToken> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: ExpressionParserToken)->Result<(), ExpressionParseError> {
        if let Some(token) = self.consume_token() {
            if token != expected {
                return Err(ExpressionParseError::General); //panic!("Expected {:?}, found {:?}", expected, token);
            }
        } else {
            return Err(ExpressionParseError::General) //panic!("Expected {:?}, but no more tokens", expected);
        }
        Ok(())
    }
}