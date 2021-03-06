use crate::inferray::InfGraph;
use crate::inferray::TripleStore;
use crate::rules::*;

/// A type alias  to unify all the rules of the reasoner
pub type Rule = fn(&mut InfGraph) -> TripleStore;
// pub trait Rule {
//     // fn specialize(&mut self, graph: std::rc::Rc<&'static InfGraph>);
//     fn fire(&mut self, graph: &mut InfGraph) -> TripleStore;
// }

/// A set of Rule, which can be aplly on a InfGraph
pub trait RuleSet {
    fn new() -> Vec<Box<Rule>>;
    // fn specialize(&mut self, graph: std::rc::Rc<&'static InfGraph>);
    fn fire_all(&mut self, graph: &mut InfGraph);
}

impl RuleSet for Vec<Box<Rule>> {
    fn new() -> Vec<Box<Rule>> {
        vec![
            Box::new(CAX_SCO),
            Box::new(CAX_EQC1),
            Box::new(CAX_EQC2),
            Box::new(SCM_EQC2),
            Box::new(PRP_INV_1_2),
            Box::new(PRP_EQP_1_2),
        ]
    }
    // fn specialize(&mut self, graph: std::rc::Rc<&'static InfGraph>) {
    // for rule in self.iter() {
    // rule.specialize(std::rc::Rc::clone(&graph));
    // }
    // }
    fn fire_all(&mut self, graph: &mut InfGraph) {
        let mut prev_size = 0;
        let mut size = graph.size();
        while prev_size != size {
            prev_size = size;
            let mut outputs = TripleStore::new();
            for rule in self.iter_mut() {
                outputs.add_all(rule(graph));
            }
            graph.dictionary.ts.add_all(outputs);
            graph.dictionary.ts.sort();
            size = graph.size();
        }
    }
}
