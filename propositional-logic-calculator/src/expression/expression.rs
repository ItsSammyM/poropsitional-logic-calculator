use std::collections::HashMap;

use super::{parser::{ExpressionParseError, ExpressionParser}, VariableNames};

#[derive(Debug, Clone)]
pub(super) struct Expression{
    node: Box<ExpressionNode>
}
impl Expression{
    pub(super) fn parse_string(input: &str)->Result<(Self, VariableNames), ExpressionParseError>{
        let mut map = HashMap::new();
        let node = ExpressionParser::parse_string(&input, &mut map)?;
        let var_names = VariableNames{
            names: map,
        };
        Ok((
            Self{
                node
            },
            var_names
        ))
    }
    pub(super) fn parse_string_with_variable_names(input: &str, variable_names: &mut VariableNames)->Result<Self, ExpressionParseError>{
        let node = ExpressionParser::parse_string(&input, &mut variable_names.names)?;
        Ok(Self{
            node
        })
    }
    pub(super) fn node_owned(self)->Box<ExpressionNode>{
        self.node
    }
}

use super::variable::Variable;

#[derive(Debug, Clone)]
pub(super) enum ExpressionNode{
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
    Variable(Variable)
}

impl ExpressionNode{
    pub(super) fn new_variable(var: Variable)->Box<Self>{Box::new(Self::Variable(var))}
    pub(super) fn new_and(a: Box<Self>, b: Box<Self>)->Box<Self>{Box::new(Self::And(a, b))}
    pub(super) fn new_or(a: Box<Self>, b: Box<Self>)->Box<Self>{Box::new(Self::Or(a, b))}
    pub(super) fn new_not(a: Box<Self>)->Box<Self>{Box::new(Self::Not(a))}

    pub(super) fn new_xor(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        let or = Self::new_or(a.clone(), b.clone());
        let and = Self::new_and(a, b);
        Self::new_and(or, Self::new_not(and))
    }
    pub(super) fn new_implies(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        Self::new_or(Self::new_not(a), b)
    }
    pub(super) fn new_biconditional(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        let or = Self::new_or(a.clone(), b.clone());
        let and = Self::new_and(a, b);
        Self::new_or(Self::new_not(or), and)
    }
}

impl ExpressionNode{
    fn traverse_owned(mut self: Box<Self>, closure: fn(Box<Self>)->Box<Self>)->Box<Self>{
        self = closure(self);
        // self = self.pushdown_not();

        //run again on children
        match *self {
            Self::And(mut a, mut b) => {
                a = a.traverse_owned(closure);
                b = b.traverse_owned(closure);
                Self::new_and(a, b)
            },
            Self::Or(mut a, mut b) => {
                a = a.traverse_owned(closure);
                b = b.traverse_owned(closure);
                Self::new_or(a, b)
            },
            Self::Variable(variable) => {Self::new_variable(variable)},
            Self::Not(mut a) => {
                a = a.traverse_owned(closure);
                Self::new_not(a)
            },
        }
    }

    // Garuntees this node is not longer NOT by pushing down the nots
    fn pushdown_not(self: Box<Self>)->Box<Self>{
        //nothing changes in this case
        let Self::Not(node) = *self else {return self};

        Box::new(match *node {
            Self::And(a, b) => {
                Self::Or(Box::new(Self::Not(a)), Box::new(Self::Not(b)))
            },
            Self::Or(a, b) => {
                Self::And(Box::new(Self::Not(a)), Box::new(Self::Not(b)))
            },
            Self::Not(a) => {*a},

            //nothing changes in this case
            Self::Variable(variable) => {Self::Not(Box::new(Self::Variable(variable)))},
        })
    }

    pub fn pushdown_not_recursive(self: Box<Self>)->Box<Self>{
        self.traverse_owned(|node: Box<Self>|node.pushdown_not())
    }

    /// Replaces A | (B & C) with (A | B) & (A | C)
    /// Replaces (B & C) | A with (B | A) & (C | A)
    /// Garuntees this node is no longer OR with AND inside
    fn distribute_or(self: Box<Self>)->Box<Self>{
        //nothing changes in this case
        let Self::Or(a, b) = *self else {return self};

        //if (a != and & b != and) {retrun self}
        if let Self::And(a_a, a_b) = *b {
            Self::new_and(
                Self::new_or(a.clone(), a_a), 
                Self::new_or(a, a_b)
            )
        }else if let Self::And(b_a, b_b) = *a {
            Self::new_and(
                Self::new_or(b_a, b.clone()),
                Self::new_or(b_b, b), 
            )
        }else{
            Self::new_or(a, b)
        }
    }
    pub fn distribute_or_recursive(self: Box<Self>)->Box<Self>{
        self.traverse_owned(|node: Box<Self>|node.distribute_or())
    }

}