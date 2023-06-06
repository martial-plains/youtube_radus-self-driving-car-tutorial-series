use std::vec;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::utils::lerp;

#[repr(C)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Level {
    pub inputs: Vec<f64>,
    pub outputs: Vec<f64>,
    pub biases: Vec<f64>,
    pub weights: Vec<Vec<f64>>,
}

impl Level {
    pub fn new(input_count: usize, output_count: usize) -> Self {
        let inputs = vec![0.0; input_count];
        let outputs = vec![0.0; output_count];
        let biases = vec![0.0; output_count];

        let mut weights = Vec::new();
        for _ in 0..input_count {
            weights.push(vec![0.0; output_count])
        }

        let mut this = Self {
            inputs,
            outputs,
            biases,
            weights,
        };

        Self::randomize(&mut this);
        this
    }

    pub fn randomize(&mut self) {
        let mut rng = thread_rng();
        for i in 0..self.inputs.len() {
            for j in 0..self.outputs.len() {
                self.weights[i][j] = rng.gen_range(0.0..1.0) * 2.0 - 1.0;
            }
        }

        for i in 0..self.biases.len() {
            self.biases[i] = rng.gen_range(0.0..1.0) * 2.0 - 1.0;
        }
    }

    pub fn feed_forward(&mut self, given_inputs: Vec<f64>) -> Vec<f64> {
        let input_len = self.inputs.len();
        self.inputs.copy_from_slice(&given_inputs[..input_len]);

        for i in 0..self.outputs.len() {
            let mut sum = 0.0;

            for j in 0..self.inputs.len() {
                sum += self.inputs[j] * self.weights[j][i];
            }

            if sum > self.biases[i] {
                self.outputs[i] = 1.0;
            } else {
                self.outputs[i] = 0.0;
            }
        }

        self.outputs.to_vec()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeuralNetwork {
    pub levels: Vec<Level>,
}

impl NeuralNetwork {
    pub fn new(neuron_counts: Vec<usize>) -> Self {
        let mut levels = Vec::new();

        for i in 0..neuron_counts.len() - 1 {
            levels.push(Level::new(neuron_counts[i], neuron_counts[i + 1]));
        }

        Self { levels }
    }

    pub fn feed_forward(&mut self, given_inputs: Vec<f64>) -> Vec<f64> {
        let mut outputs = self.levels[0].feed_forward(given_inputs);
        for i in 1..self.levels.len() {
            outputs = self.levels[i].feed_forward(outputs);
        }

        outputs
    }

    pub fn mutate(&mut self, amount: Option<f64>) {
        let mut rng = thread_rng();
        let amount = amount.unwrap_or(1.0);

        self.levels.iter_mut().for_each(|level| {
            for i in 0..level.biases.len() {
                level.biases[i] = lerp(level.biases[i], rng.gen_range(0.0..1.0) * 2.0 - 1.0, amount)
            }

            for i in 0..level.weights.len() {
                for j in 0..level.weights[i].len() {
                    level.weights[i][j] = lerp(
                        level.weights[i][j],
                        rng.gen_range(0.0..1.0) * 2.0 - 1.0,
                        amount,
                    )
                }
            }
        });
    }
}
