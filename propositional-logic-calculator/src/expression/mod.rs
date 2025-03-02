use std::collections::HashMap;

use expression::Expression;
use parser::ExpressionParseError;
use variable::Variable;

mod parser;
mod variable;
mod knowledge_base;
mod expression;

pub use knowledge_base::KnowledgeBase;



pub struct Workspace{
    variable_names: VariableNames,
    expression_set: ExpressionSet
}
impl Workspace{
    pub fn new()->Self{
        Self { variable_names: VariableNames::new(), expression_set: ExpressionSet::new() }
    }
    pub fn parse_expression(&mut self, input: &str)->Result<(), ExpressionParseError>{
        match Expression::parse_string_with_variable_names(input, &mut self.variable_names) {
            Ok(expr) => {
                self.expression_set.push(expr);
                Ok(())
            },
            Err(e) => {Err(e)},
        }
    }
    pub fn knowledge_base_from_all_expressions(&self)->KnowledgeBase{
        let mut kb = self.expression_set.set.iter()
            .fold(KnowledgeBase::tautology(), |mut kb, x|{
                kb.combine(KnowledgeBase::from_expression(x.clone()));
                kb
            });
        println!("Knowledge base complete");
        kb.simplify();
        println!("Knowledge base simplified");
        kb
    }
    pub fn print_knowledge_base_from_all_expressions(&self){
        println!("{}", self.display_knowledge_base())
    }
    fn display_knowledge_base(&self) -> String {
        self.knowledge_base_from_all_expressions().display(&self.variable_names)
    }
}


pub(super) struct VariableNames{
    names: HashMap<String, Variable>
}
impl VariableNames{
    fn new()->Self{
        Self { names: HashMap::new() }
    }
    fn push_names(&mut self, names: HashMap<String, Variable>){
        self.names.extend(names.into_iter());
    }
    fn names_map_owned(self)->HashMap<String, Variable>{
        self.names
    }
    fn get_name_from_variable(&self, var: &Variable)->Option<&String>{
        self.names
            .iter()
            .find(|(_, v)|**v==*var)
            .map(|(s,_)|s)
    }
    fn get_variable(&self, var: &String)->Option<&Variable>{
        self.names.get(var)
    }
}

pub(super) struct ExpressionSet{
    set: Vec<Expression>
}
impl ExpressionSet{
    pub(super) fn new()->Self{
        Self{set: Vec::new()}
    }
    pub(super) fn push(&mut self, expr: Expression){
        self.set.push(expr);
    }
}