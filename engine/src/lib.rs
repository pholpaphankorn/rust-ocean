use wasm_bindgen::prelude::*;

pub mod physics;
pub mod waves;

use physics::{FluidGrid, GRID};
use waves::{gerstner_stack, WAVE_SCALE};

#[wasm_bindgen]
pub struct SimState {
    grid: FluidGrid,
    time: f32,
}

#[wasm_bindgen]
impl SimState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SimState {
            grid: FluidGrid::new(),
            time: 0.0,
        }
    }

    pub fn step(&mut self) {
        self.grid.compute_step(self.time);
        self.time += 0.1; // Matches internal DT advancement increments cleanly
    }

    pub fn grid_size(&self) -> usize {
        GRID
    }

    pub fn get_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::with_capacity(GRID * GRID * 4);
        let half = GRID as f32 / 2.0;
        let mean = self.grid.eta.iter().sum::<f32>() / self.grid.eta.len() as f32;

        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 - half;
                let pz = z as f32 - half;

                // Extraction logic remains compatible with frontend strides
                let swe_y = self.grid.eta[z * GRID + x] - mean;
                let (gdx, gdy, gdz) = gerstner_stack(px * WAVE_SCALE, pz * WAVE_SCALE, self.time);
                let energy = 1.0 + swe_y.abs() * 1.5;

                vertices.push(px + gdx * energy);
                vertices.push(swe_y + gdy * energy);
                vertices.push(pz + gdz * energy);
                vertices.push(swe_y); 
            }
        }
        vertices
    }

    pub fn get_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let g = GRID as u32;
        for z in 0..(g - 1) {
            for x in 0..(g - 1) {
                let i = z * g + x;
                indices.push(i);
                indices.push(i + g);
                indices.push(i + 1);
                indices.push(i + 1);
                indices.push(i + g);
                indices.push(i + g + 1);
            }
        }
        indices
    }

    pub fn splash(&mut self, x: usize, z: usize, amount: f32) {
        self.grid.add_splash(x, z, amount);
    }
}