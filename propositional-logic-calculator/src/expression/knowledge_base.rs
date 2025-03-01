use super::{expression::ExpressionNode, variable::Variable, Expression, VariableNames};

#[derive(Debug)]
pub struct KnowledgeBase{
    facts: Vec<KnowledgeBaseFact>
}
impl KnowledgeBase{
    pub(super) fn tautology()->Self{
        Self{facts: Vec::new()}
    }
    fn facts(&self)->&Vec<KnowledgeBaseFact>{
        &self.facts
    }
    pub(super) fn from_expression(expression: Expression)->Self{
        KnoweldgeBaseBuilder::from_expression(expression)
    }
    pub(super) fn combine(&mut self, other: KnowledgeBase){
        self.facts.extend(other.facts.into_iter());
    }
    fn push_fact(&mut self, fact: KnowledgeBaseFact){
        self.facts.push(fact);
    }
    pub(super) fn display(&self, vars: &VariableNames)->String{
        let mut out: String = String::new();

        for fact in self.facts(){
            out.push('[');
            for (i, literal) in fact.literals().iter().enumerate(){
                let Some(var_name) = vars.get_name_from_variable(&literal.var()) else {unreachable!()};
                if literal.not() {
                    out.push('!');
                }
                out.push_str(format!("{}", var_name).as_str());
                if i != fact.literals().len().saturating_sub(1) {
                    out.push_str(", ");
                }
            }
            out.push_str("]\n");
        }
        out
    }
    pub(super) fn simplify(&mut self){
        /*
            Implies
            [x]
            [!x, y]
            ->
            [y]
            !x is false so y must be true

            
            Remove supersets
            [y]
            [!x, y]
            we already know !x|y because we know y so we can remove [!x, y]
        */

    }
}


#[derive(Debug)]
struct KnowledgeBaseFact{
    set: Vec<KnowledgeBaseLiteral>
}
impl KnowledgeBaseFact{
    fn new(set: Vec<KnowledgeBaseLiteral>)->Self{
        Self{set}
    }
    pub(super) fn literals(&self)->&Vec<KnowledgeBaseLiteral>{
        &self.set
    }
}


#[derive(Clone, Debug)]
struct KnowledgeBaseLiteral{
    not: bool,
    var: Variable
}
impl KnowledgeBaseLiteral{
    fn new(not: bool, var: Variable)->Self{
        Self{not, var}
    }
    pub(super) fn not(&self)->bool{
        self.not
    }
    pub(super) fn var(&self)->&Variable{
        &self.var
    }
}


struct KnoweldgeBaseBuilder{
    state: KnowledgeBaseFactBuilder,
    base: KnowledgeBase
}
impl KnoweldgeBaseBuilder{
    fn from_expression(expression: Expression)->KnowledgeBase{
        let cnf_node = expression.node_owned().pushdown_not_recursive().distribute_or_recursive();
        
        let mut builder = Self{
            state: KnowledgeBaseFactBuilder::None,
            base: KnowledgeBase {
                facts: Vec::new()
            }
        };

        builder.from_expression_recursive(cnf_node);
        builder.base
    }
    fn from_expression_recursive(&mut self, expr: Box<ExpressionNode>){
        match *expr {
            ExpressionNode::And(a, b) => {
                self.from_expression_recursive(a);
                self.from_expression_recursive(b);
            },
            ExpressionNode::Or(a, b) => {
                let already_in_fact = matches!(self.state, KnowledgeBaseFactBuilder::Fact(_));
                if !already_in_fact{
                    let fact = Vec::new();
                    self.state = KnowledgeBaseFactBuilder::Fact(fact);
                }
                
                self.from_expression_recursive(a);
                self.from_expression_recursive(b);

                if !already_in_fact {
                    let KnowledgeBaseFactBuilder::Fact(fact) = &mut self.state else {panic!()};
                    self.base.push_fact(KnowledgeBaseFact::new(fact.clone()));
                    self.state = KnowledgeBaseFactBuilder::None;
                }
            },
            ExpressionNode::Variable(variable) => {
                let new_fact = KnowledgeBaseLiteral::new(false, variable);
                if let KnowledgeBaseFactBuilder::Fact(fact) = &mut self.state{
                    fact.push(new_fact);
                }else{
                    let mut fact: Vec<KnowledgeBaseLiteral> = Vec::new();
                    fact.push(new_fact);
                    self.base.push_fact(KnowledgeBaseFact::new(fact));
                }
            },
            ExpressionNode::Not(a) => {

                let ExpressionNode::Variable(variable) = *a else {panic!()};
                let new_fact = KnowledgeBaseLiteral::new(true, variable);

                if let KnowledgeBaseFactBuilder::Fact(fact) = &mut self.state{
                    fact.push(new_fact);
                }else{
                    let mut fact: Vec<KnowledgeBaseLiteral> = Vec::new();
                    fact.push(new_fact);
                    self.base.push_fact(KnowledgeBaseFact::new(fact));
                }
            },
        }
    }
}

#[derive(Default)]
enum KnowledgeBaseFactBuilder{
    Fact(Vec<KnowledgeBaseLiteral>),
    #[default]
    None
}