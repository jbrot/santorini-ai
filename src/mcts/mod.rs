use rand::Rng;

mod node;
pub use node::Node;

pub mod tree_policy;
pub use tree_policy::TreePolicy;

pub mod santorini;

pub trait Simulation<T, R: Rng>: Send {
    fn simulate(&self, state: &T, rng: &mut R) -> f64;
}

pub trait Expansion<T>: Send {
    fn expand(&self, state: &T) -> Vec<T>;
}

pub struct MctsParams<T, R: Rng> {
    pub tree_policy: Box<dyn TreePolicy<T>>,
    pub simulation: Box<dyn Simulation<T, R>>,
    pub expansion: Box<dyn Expansion<T>>,
    pub rng: R,
    pub budget: u32,
}

impl<T, R: Rng> MctsParams<T, R> {
    pub fn new<S, E>(simulation: S, expansion: E, rng: R) -> Self
    where
        S: 'static + Simulation<T, R>,
        E: 'static + Expansion<T>,
    {
        MctsParams {
            tree_policy: Box::new(tree_policy::UCB1::default()),
            simulation: Box::new(simulation),
            expansion: Box::new(expansion),
            rng,
            budget: 500,
        }
    }

    pub fn tree_policy<P: 'static + TreePolicy<T>>(self, tree_policy: P) -> Self {
        MctsParams {
            tree_policy: Box::new(tree_policy),
            ..self
        }
    }

    pub fn budget(self, budget: u32) -> Self {
        MctsParams { budget, ..self }
    }
}

pub struct Mcts<T, R: Rng> {
    pub params: MctsParams<T, R>,
    pub root_node: Node<T>,
}

impl<T, R: Rng> Mcts<T, R> {
    pub fn new(mut params: MctsParams<T, R>, root_node: T) -> Self {
        let root_node = Node::new(&mut params, root_node);
        Mcts { params, root_node }
    }

    pub fn advance(&mut self) {
        for _ in 0..self.params.budget {
            self.root_node.step(&mut self.params);
        }

        let children = self
            .root_node
            .children
            .as_ref()
            .expect("Root node missing children");
        assert!(children.len() > 0, "Root node has no children!");

        let mut best_score = children[0].score as f64 / children[0].iterations as f64;
        let mut best_score_idx = 0;

        // let mut most_visits = children[0].iterations;
        // let mut most_visits_idx = 0;

        for (index, child) in children.iter().enumerate() {
            if child.score > best_score {
                best_score = child.score;
                best_score_idx = index;
            }

            // if child.iterations > most_visits {
            //     most_visits = child.iterations;
            //     most_visits_idx = index;
            // }
        }

        take_mut::take(&mut self.root_node, |node| {
            node.children
                .unwrap()
                .into_iter()
                .nth(best_score_idx)
                .expect("Invalid best child index!")
        });
    }
}
