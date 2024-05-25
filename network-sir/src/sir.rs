use crate::{Graph, GraphAdjacencyList};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
enum SIRState {
    Susceptible,
    Infected,
    Recovered,
}

#[derive(Debug)]
pub struct SIRGraph<'a> {
    graph: &'a GraphAdjacencyList,
    infected1: Vec<SIRState>,
    infected2: Vec<SIRState>,
    first_is_current: bool,
    n_infected: usize,
    n_susceptible: usize,
    n_recovered: usize,

    // much quicker to just keep track of the number of infected neighbors
    n_infected_neighbors: Vec<usize>,
    recovery_rate: f64,
    infection_rate: f64,
    initial_infection_probability: f64,
    rng: rand::rngs::ThreadRng,

    // social distancing settings
    enforce_social_distancing: bool,
    max_contacts: usize,
}

impl<'a> SIRGraph<'a> {
    pub fn n_infected(&self) -> usize {
        self.n_infected
    }

    pub fn n_susceptible(&self) -> usize {
        self.n_susceptible
    }

    pub fn n_recovered(&self) -> usize {
        self.n_recovered
    }

    pub fn n_nodes(&self) -> usize {
        self.graph.n_nodes()
    }

    pub fn prop_infected(&self) -> f64 {
        self.n_infected as f64 / self.graph.n_nodes() as f64
    }

    pub fn reset(&mut self, infection_rate: f64, recovery_rate: f64) {
        self.recovery_rate = recovery_rate;
        self.infection_rate = infection_rate;
        self.n_infected = 0;
        self.n_susceptible = 0;
        self.n_recovered = 0;
        for i in 0..self.graph.n_nodes() {
            if self.rng.gen::<f64>() < self.initial_infection_probability {
                self.infected1[i] = SIRState::Infected;
                self.n_infected += 1;
            } else {
                self.infected1[i] = SIRState::Susceptible;
                self.n_susceptible += 1;
            }
            self.n_infected_neighbors[i] = 0;
        }

        for i in 0..self.graph.n_nodes() {
            if matches!(self.infected1[i], SIRState::Infected) {
                for j in self.graph.nodes[i].iter() {
                    self.n_infected_neighbors[*j] += 1;
                }
            }
        }
        self.first_is_current = true;
    }

    pub fn from_graph_with_sd(
        graph: &'a GraphAdjacencyList,
        recovery_rate: f64,
        infection_rate: f64,
        initial_infection_probability: f64,
        max_contacts: usize,
    ) -> Self {
        let mut g = Self::from_graph(
            graph,
            recovery_rate,
            infection_rate,
            initial_infection_probability,
        );
        g.enforce_social_distancing = true;
        g.max_contacts = max_contacts;
        g
    }

    pub fn from_graph(
        graph: &'a GraphAdjacencyList,
        recovery_rate: f64,
        infection_rate: f64,
        initial_infection_probability: f64,
    ) -> Self {
        let n_nodes = graph.n_nodes();
        let infected1 = vec![SIRState::Susceptible; n_nodes];
        let infected2 = vec![SIRState::Susceptible; n_nodes];
        let n_infected_neighbors = vec![0; n_nodes];

        let rng = rand::thread_rng();

        let mut g = Self {
            graph,
            infected1,
            infected2,
            first_is_current: true,
            n_infected_neighbors,
            n_infected: 0,
            n_susceptible: 0,
            n_recovered: 0,
            recovery_rate,
            infection_rate,
            initial_infection_probability,
            rng,
            enforce_social_distancing: false,
            max_contacts: 0,
        };
        g.reset(infection_rate, recovery_rate);
        g
    }

    pub fn step(&mut self) {
        let (infected, infected_next) = if self.first_is_current {
            (&mut self.infected1, &mut self.infected2)
        } else {
            (&mut self.infected2, &mut self.infected1)
        };

        for i in 0..self.graph.n_nodes() {
            infected_next[i] = infected[i];
            match infected[i] {
                SIRState::Recovered => {}
                SIRState::Susceptible => {
                    let n_infected_neighbors = self.n_infected_neighbors[i];
                    let n_neighbors = self.graph.nodes[i].len();
                    let infection_prop = if self.enforce_social_distancing
                        && self.max_contacts < n_neighbors
                    {
                        // randomly sample max_contacts neighbors
                        let mut n_sampled_neighbors_infected = 0;
                        let mut infected_neighbors = n_infected_neighbors;
                        let mut total_neighbors = self.graph.nodes[i].len();
                        for _ in 0..self.max_contacts {
                            let j = self.rng.gen_range(0..total_neighbors);
                            if j < infected_neighbors {
                                infected_neighbors -= 1;
                                n_sampled_neighbors_infected += 1;
                            }
                            total_neighbors -= 1;
                        }
                        1.0 - (1.0 - self.infection_rate).powf(n_sampled_neighbors_infected as f64)
                    } else {
                        1.0 - (1.0 - self.infection_rate).powf(n_infected_neighbors as f64)
                    };
                    if self.rng.gen::<f64>() < infection_prop {
                        infected_next[i] = SIRState::Infected;
                        for j in self.graph.nodes[i].iter() {
                            self.n_infected_neighbors[*j] += 1;
                        }
                        self.n_infected += 1;
                        self.n_susceptible -= 1;
                    }
                }
                SIRState::Infected => {
                    if self.rng.gen::<f64>() < self.recovery_rate {
                        infected_next[i] = SIRState::Recovered;
                        for j in self.graph.nodes[i].iter() {
                            self.n_infected_neighbors[*j] -= 1;
                        }
                        self.n_infected -= 1;
                        self.n_recovered += 1;
                    }
                }
            }
        }

        debug_assert!(
            self.n_infected
                == infected_next
                    .iter()
                    .filter(|&&x| matches!(x, SIRState::Infected))
                    .count(),
        );
        debug_assert!(
            self.n_susceptible
                == infected_next
                    .iter()
                    .filter(|&&x| matches!(x, SIRState::Susceptible))
                    .count()
        );
        debug_assert!(
            self.n_recovered
                == infected_next
                    .iter()
                    .filter(|&&x| matches!(x, SIRState::Recovered))
                    .count()
        );
        assert!(self.n_infected + self.n_susceptible + self.n_recovered == self.graph.n_nodes());
        self.first_is_current = !self.first_is_current;
    }
}
