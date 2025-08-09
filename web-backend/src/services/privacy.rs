use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Laplace, Normal};

#[derive(Debug, Clone, Copy)]
pub enum Mechanism {
    Laplace { sensitivity: f64 },
    Gaussian { sensitivity: f64, delta: f64 },
    RandomizedResponse { p: f64 },
}

#[derive(Debug, Clone)]
pub struct PrivacyBudget {
    pub epsilon_total: f64,
    pub delta_total: f64,
    pub epsilon_spent: f64,
    pub delta_spent: f64,
}

impl PrivacyBudget {
    pub fn new(epsilon_total: f64, delta_total: f64) -> Self {
        Self {
            epsilon_total,
            delta_total,
            epsilon_spent: 0.0,
            delta_spent: 0.0,
        }
    }

    pub fn remaining(&self) -> (f64, f64) {
        (
            self.epsilon_total - self.epsilon_spent,
            self.delta_total - self.delta_spent,
        )
    }

    pub fn spend(&mut self, epsilon: f64, delta: f64) -> bool {
        let (e_rem, d_rem) = self.remaining();
        if epsilon <= e_rem + f64::EPSILON && delta <= d_rem + f64::EPSILON {
            self.epsilon_spent += epsilon.max(0.0);
            self.delta_spent += delta.max(0.0);
            true
        } else {
            false
        }
    }
}

pub struct DifferentialPrivacyEngine {
    pub budget: PrivacyBudget,
}

impl DifferentialPrivacyEngine {
    pub fn new(epsilon: f64, delta: f64) -> Self {
        Self {
            budget: PrivacyBudget::new(epsilon.max(0.0), delta.max(0.0)),
        }
    }

    pub fn add_noise(&mut self, true_value: f64, mechanism: Mechanism, epsilon_cost: f64) -> f64 {
        match mechanism {
            Mechanism::Laplace { sensitivity } => {
                if !self.budget.spend(epsilon_cost, 0.0) {
                    return f64::NAN;
                }
                let scale = (sensitivity.max(f64::EPSILON)) / epsilon_cost.max(f64::EPSILON);
                let dist = Laplace::new(0.0, scale).unwrap();
                let mut rng = thread_rng();
                true_value + dist.sample(&mut rng)
            }
            Mechanism::Gaussian { sensitivity, delta } => {
                if !self.budget.spend(epsilon_cost, delta) {
                    return f64::NAN;
                }
                // Analytical Gaussian mechanism sigma ~ sqrt(2 ln(1.25/delta)) * sensitivity / epsilon
                let sigma = (2.0 * (1.25 / delta.max(1e-12)).ln()).sqrt()
                    * sensitivity.max(f64::EPSILON)
                    / epsilon_cost.max(f64::EPSILON);
                let dist = Normal::new(0.0, sigma.max(f64::EPSILON)).unwrap();
                let mut rng = thread_rng();
                true_value + dist.sample(&mut rng)
            }
            Mechanism::RandomizedResponse { p } => {
                if !self.budget.spend(epsilon_cost, 0.0) {
                    return f64::NAN;
                }
                // For RR, assume true_value is in [0,1] probability estimate; flip towards 0/1 with p
                let mut rng = thread_rng();
                let flip: bool = rng.gen::<f64>() < p;
                if flip {
                    if rng.gen::<bool>() {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    true_value
                }
            }
        }
    }

    pub fn clip_values(values: &mut [f64], min_value: f64, max_value: f64) {
        for v in values.iter_mut() {
            if *v < min_value {
                *v = min_value;
            }
            if *v > max_value {
                *v = max_value;
            }
        }
    }

    pub fn spend_only(&mut self, epsilon_cost: f64, delta_cost: f64) -> bool {
        self.budget.spend(epsilon_cost, delta_cost)
    }
}
