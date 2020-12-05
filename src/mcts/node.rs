use super::MctsParams;
use rand::Rng;

#[derive(Clone)]
pub struct Node<T> {
    pub children: Option<Vec<Node<T>>>,
    pub iterations: u32,
    pub score: f64,
    pub state: T,
}

impl<T> Node<T> {
    pub fn new<R: Rng>(params: &mut MctsParams<T, R>, state: T) -> Self {
        let score = params.simulation.simulate(&state, &mut params.rng);
        Node {
            children: None,
            iterations: 1,
            score,
            state,
        }
    }

    pub fn expand<R: Rng>(&mut self, params: &mut MctsParams<T, R>) -> (u32, f64) {
        assert!(self.children.is_none(), "Node has already been expanded!");

        let mut children = Vec::new();
        let mut new_scores: f64 = 0.0;
        for child in params.expansion.expand(&self.state) {
            let node = Node::new(params, child);
            new_scores += -1.0 * node.score;
            children.push(node);
        }

        let new_nodes = children.len() as u32;
        let new_score = self.score * (self.iterations as f64) + new_scores;
        self.iterations += new_nodes;
        self.score = new_score / (self.iterations as f64);
        self.children = Some(children);

        (new_nodes, new_scores)
    }

    pub fn step<R: Rng>(&mut self, params: &mut MctsParams<T, R>) -> (u32, f64) {
        match self.children.as_ref() {
            None => self.expand(params),
            Some(children) => {
                if children.len() == 0 {
                    (0, 0.0)
                } else {
                    let immutable_children: &Vec<Node<T>> = &*children;
                    let idx = params.tree_policy.select(self, immutable_children);

                    let (count, delta) = self.children.as_mut().unwrap()[idx].step(params);
                    let new_score = self.score * self.iterations as f64 - delta;
                    self.iterations += count;
                    self.score = new_score / (self.iterations as f64);
                    (count, -delta)
                }
            }
        }
    }
}
