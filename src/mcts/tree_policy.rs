use super::Node;

pub trait TreePolicy<T>: Send {
    fn select(&self, parent: &Node<T>, children: &Vec<Node<T>>) -> usize;
}

pub struct UCB1 {
    pub parameter: f64,
}

impl UCB1 {
    pub fn default() -> UCB1 {
        UCB1 {
            parameter: f64::sqrt(2.0),
        }
    }
}

impl<T> TreePolicy<T> for UCB1 {
    fn select(&self, parent: &Node<T>, children: &Vec<Node<T>>) -> usize {
        let mut best_index = None;
        let mut best_weight = None;
        for (index, child) in children.iter().enumerate() {
            // Rescale to be between 0 and 1
            let child_score = (1.0 + child.score) / 2.0;

            let augment = f64::ln(parent.iterations as f64);
            let augment = augment / (child.iterations as f64);
            let augment = f64::sqrt(augment);

            let weight = child_score + self.parameter * augment;
            match best_weight {
                None => {
                    best_weight = Some(weight);
                    best_index = Some(index);
                }
                Some(best) => {
                    if weight > best {
                        best_weight = Some(weight);
                        best_index = Some(index);
                    }
                }
            }
        }

        best_index.expect("No children!")
    }
}

pub struct PUCT {
    pub parameter: f64,
}

impl<T> TreePolicy<T> for PUCT {
    fn select(&self, parent: &Node<T>, children: &Vec<Node<T>>) -> usize {
        let mut best_index = None;
        let mut best_weight = None;
        for (index, child) in children.iter().enumerate() {
            // Rescale to be between 0 and 1
            let child_score = (1.0 + child.score) / 2.0;

            let augment = f64::sqrt(parent.iterations as f64);
            let augment = augment / (child.iterations as f64);
            let weight = child_score + self.parameter * augment;
            match best_weight {
                None => {
                    best_weight = Some(weight);
                    best_index = Some(index);
                }
                Some(best) => {
                    if weight > best {
                        best_weight = Some(weight);
                        best_index = Some(index);
                    }
                }
            }
        }

        best_index.expect("No children!")
    }
}
