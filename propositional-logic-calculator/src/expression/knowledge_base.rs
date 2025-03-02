use std::collections::HashMap;
use std::hash::Hash;
use super::{expression::ExpressionNode, variable::Variable, Expression, VariableNames};

/// Empty KB is tautology
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
    /**
        Remove Unit propagation
        [x], [!x, y] becomes [x], [y]

        Remove supersets
        [y], [!x, y] becomes [y]

        Remove duplicates
        [x], [x] becomes [x]

        Remove tautologies
        [x, !x] becomes empty


        !!! Still missing
        !!! Leo says this is impossible according to his professor (The guy who like invented SAT solvers or something)
        [x, z], [!x, y] [!z, y] becomes [x, z], [y]
        For this we need a to just add [!y] to a KB and check for contradictions
    */
    pub(super) fn simplify(&mut self){

        let mut run_again = true;

        let mut subset_cache: HashMap<(KnowledgeBaseFact, KnowledgeBaseFact), bool> = HashMap::new();

        let cache_ref = &mut subset_cache;

        let mut is_subset = move |a: &KnowledgeBaseFact, b: &KnowledgeBaseFact|{
            let tuple = (a.clone(),b.clone());
            if let Some(is_subset) = cache_ref.get(&tuple) {
                *is_subset
            }else{
                let is_subset = tuple.0.is_subset(&tuple.1);
                cache_ref.insert(tuple, is_subset);
                is_subset
            }
        };

        
        let mut tautology_cache: HashMap<KnowledgeBaseFact, bool> = HashMap::new();
        let tautology_cache_ref = &mut tautology_cache;
        let mut is_tautology = move |a: &KnowledgeBaseFact|{
            if let Some(a) = tautology_cache_ref.get(&a) {
                *a
            }else{
                let tautology = a.tautology();
                tautology_cache_ref.insert(a.clone(), tautology);
                tautology
            }
        };

        while run_again {
            run_again = false;

            let mut out = Vec::new();
            let mut units = Vec::new();

            //remove superset & duplicates & tautologies
            for fact_a in self.facts.iter(){

                if out.contains(fact_a) || is_tautology(fact_a) {
                    run_again = true;
                    continue;
                }

                if !self.facts
                    .iter()
                    .any(|fact_b|{
                        is_subset(fact_b, fact_a) && !is_subset(fact_a, fact_b)
                        // fact_b.is_subset(fact_a) && !fact_a.is_subset(fact_b)
                    })
                {
                    if let Some(literal) = fact_a.is_unit() {
                        units.push(literal);
                    }
                    out.push(fact_a.clone());
                }else{
                    run_again = true;
                }
            }

            self.facts = out
                .into_iter()
                .map(|mut fact|{
                    if fact.filter_negative_literals(&units) {
                        run_again = true;
                    }
                    fact
                })
                .collect();
        }
    }
}


/// empty fact is contradiction
#[derive(Debug, Clone, Eq)]
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
    fn contains(&self, literal: &KnowledgeBaseLiteral)->bool{
        self.set.contains(literal)
    }
    fn is_subset(&self, other: &KnowledgeBaseFact)->bool{
        self.set.iter()
            .all(|literal|other.contains(literal))
    }
    fn tautology(&self)->bool{
        self.set.iter().any(|x|
            self.set.iter().any(|y|
                x.complement_of(y)
            )
        )
    }
    fn contradiction(&self)->bool{
        if self.set.is_empty() {return true;}

        self.set.iter().enumerate().any(|(i,x)|{
            self.set.iter().skip(i).any(|y|{
                x.complement_of(y)
            })
        })
    }
    fn filter_literal(&mut self, literal: &KnowledgeBaseLiteral){
        self.set.retain(|l|l!=literal);
    }
    /// if any element was removed then true
    fn filter_negative_literals(&mut self, literals: &Vec<&KnowledgeBaseLiteral>)->bool{
        let mut changed = false;
        self.set.retain(|l|{
            if !literals.contains(&&l.negated()) {
                true
            }else{
                changed = true;
                false
            }
        });
        changed
    }
    // fn remove_duplicates(&mut self){
    //     self.set = self.set.iter()
    //         .enumerate()
    //         .filter(|(i, x)|
    //             self.set.iter().skip(i).any(|y|{
    //                 x.complement_of(y)
    //             })
    //         )
    // }
    fn is_unit(&self)->Option<&KnowledgeBaseLiteral>{
        if self.set.len() != 1 {
            None
        } else {
            self.set.first()
        }
    }
}
impl PartialEq for KnowledgeBaseFact{
    fn eq(&self, other: &Self) -> bool {
        self.is_subset(other) && other.is_subset(self)
    }
}
impl Hash for KnowledgeBaseFact{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set.len().hash(state);
        self.set.first().hash(state)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    fn complement_of(&self, other: &KnowledgeBaseLiteral)->bool{
        self.not == !other.not && self.var == other.var
    }
    fn negated(&self)->Self{
        let mut x = self.clone();
        x.not = !x.not;
        x
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