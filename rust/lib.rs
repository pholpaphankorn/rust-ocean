use wasm_bindgen::prelude::*;

const GRID: usize = 128;
const DT: f32 = 0.1;
const G: f32 = 9.8;
const DX: f32 = 1.0;
const H: f32 = 1.0;
const DAMPING: f32 = 0.995;

// Scale factor: maps grid coords → wave space
// Smaller = longer waves relative to grid
const WAVE_SCALE: f32 = 0.08;

// ─── Gerstner Wave ────────────────────────────────────────────────────────────

struct GerstnerWave {
    amplitude: f32,
    frequency: f32,
    speed: f32,
    dir_x: f32,
    dir_z: f32,
    steepness: f32,
}

impl GerstnerWave {
    fn displace(&self, x: f32, z: f32, time: f32) -> (f32, f32, f32) {
        let dot = self.dir_x * x + self.dir_z * z;
        let phase = self.frequency * dot + self.speed * time;

        let dy = self.amplitude * phase.sin();
        let dx = self.steepness * self.amplitude * self.dir_x * phase.cos();
        let dz = self.steepness * self.amplitude * self.dir_z * phase.cos();

        (dx, dy, dz)
    }
}

// Four stacked waves — different scales and directions
fn gerstner_stack(x: f32, z: f32, time: f32) -> (f32, f32, f32) {
    let waves: [GerstnerWave; 4] = [
        // long rolling swell from the west
        GerstnerWave {
            amplitude: 0.8,
            frequency: 0.15,
            speed: 1.2,
            dir_x: 1.0,
            dir_z: 0.0,
            steepness: 0.5,
        },
        // medium chop from the southwest
        GerstnerWave {
            amplitude: 0.35,
            frequency: 0.3,
            speed: 1.8,
            dir_x: 0.7,
            dir_z: 0.7,
            steepness: 0.4,
        },
        // cross-chop from the south
        GerstnerWave {
            amplitude: 0.18,
            frequency: 0.55,
            speed: 2.4,
            dir_x: 0.2,
            dir_z: 0.98,
            steepness: 0.3,
        },
        // fine ripples from the northwest
        GerstnerWave {
            amplitude: 0.07,
            frequency: 1.1,
            speed: 3.5,
            dir_x: -0.7,
            dir_z: 0.7,
            steepness: 0.2,
        },
    ];

    let (mut tdx, mut tdy, mut tdz) = (0.0_f32, 0.0_f32, 0.0_f32);
    for wave in &waves {
        let (dx, dy, dz) = wave.displace(x, z, time);
        tdx += dx;
        tdy += dy;
        tdz += dz;
    }
    (tdx, tdy, tdz)
}

// Returns only the height (Y) at a boundary point — used to drive SWE edges
fn gerstner_height(x: f32, z: f32, time: f32) -> f32 {
    let (_, dy, _) = gerstner_stack(x, z, time);
    dy
}

// Fast deterministic noise — looks random but is pure math
// Returns a value in [-1, 1]
fn noise(x: f32, t: f32) -> f32 {
    let n = (x * 127.1 + t * 311.7).sin() * 43758.545;
    n - n.floor() - 0.5 // fract mapped to [-0.5, 0.5] * 2
}

// Slowly varying amplitude envelope per x position
// Makes some areas calmer, some wilder, shifting over time
fn amplitude_envelope(x: f32, t: f32) -> f32 {
    let slow = (x * 0.3 + t * 0.07).sin(); // slow spatial variation
    let drift = (x * 0.11 + t * 0.03).cos(); // even slower drift
                                             // map from [-1,1] to [0.3, 1.7] — never fully calm, never too wild
    0.3 + (slow + drift + 2.0) * 0.35
}

// ─── Simulation State ─────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct SimState {
    eta: Vec<f32>, // SWE surface deviation
    u: Vec<f32>,   // x-velocity on x-faces (staggered)
    v: Vec<f32>,   // z-velocity on z-faces (staggered)
    time: f32,     // elapsed simulation time
}

#[wasm_bindgen]
impl SimState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let size = GRID * GRID;

        // Start flat — Gerstner boundary will build the ocean naturally
        SimState {
            eta: vec![0.0; size],
            u: vec![0.0; size],
            v: vec![0.0; size],
            time: 0.0,
        }
    }

    pub fn step(&mut self) {
        // ── Step 1: Apply Gerstner boundary conditions BEFORE physics ──────────
        // Drive all 4 edges with Gerstner wave heights.
        // SWE sees these as "incoming wave energy" and propagates them inward.
        let half = GRID as f32 / 2.0;

        // ── Chaotic multi-edge wave generation ────────────────────────────────────

        // North edge (z=0): main long swell traveling south
        for x in 0..GRID {
            let px = (x as f32 - half) * WAVE_SCALE;
            let base = gerstner_height(px, 0.0, self.time);
            let f1 = (px * 2.3 + self.time * 0.97).sin() * 0.4;
            let f2 = (px * 5.1 + self.time * 1.73).sin() * 0.2;
            let f3 = (px * 0.7 + self.time * 0.41).sin() * 0.6;
            let envelope = amplitude_envelope(x as f32, self.time);
            let micro = noise(x as f32, self.time) * 0.15;
            self.eta[0 * GRID + x] = (base + f1 + f2 + f3 + micro) * envelope;
        }

        // South edge (z=GRID-1): storm chop traveling north
        // different frequencies → never syncs with north edge
        for x in 0..GRID {
            let px = (x as f32 - half) * WAVE_SCALE;
            let f1 = (px * 3.1 + self.time * 1.13).sin() * 0.5;
            let f2 = (px * 1.7 + self.time * 0.67).sin() * 0.3;
            let f3 = (px * 6.3 + self.time * 2.11).sin() * 0.15;
            let envelope = amplitude_envelope(x as f32 + 50.0, self.time + 7.3); // offset → different phase
            let micro = noise(x as f32 + 100.0, self.time) * 0.12;
            self.eta[(GRID - 1) * GRID + x] = (f1 + f2 + f3 + micro) * envelope;
        }

        // West edge (x=0): cross-swell traveling east
        for z in 0..GRID {
            let pz = (z as f32 - half) * WAVE_SCALE;
            let f1 = (pz * 1.9 + self.time * 0.83).sin() * 0.45;
            let f2 = (pz * 4.3 + self.time * 1.51).sin() * 0.18;
            let f3 = (pz * 0.9 + self.time * 0.37).sin() * 0.5;
            let envelope = amplitude_envelope(z as f32 + 25.0, self.time + 3.1);
            let micro = noise(z as f32 + 200.0, self.time) * 0.1;
            self.eta[z * GRID + 0] = (f1 + f2 + f3 + micro) * envelope;
        }

        // East edge (x=GRID-1): diagonal chop traveling west
        for z in 0..GRID {
            let pz = (z as f32 - half) * WAVE_SCALE;
            let f1 = (pz * 2.7 + self.time * 1.29).sin() * 0.35;
            let f2 = (pz * 0.5 + self.time * 0.53).sin() * 0.55;
            let f3 = (pz * 7.1 + self.time * 1.97).sin() * 0.12;
            let envelope = amplitude_envelope(z as f32 + 75.0, self.time + 11.7);
            let micro = noise(z as f32 + 300.0, self.time) * 0.1;
            self.eta[z * GRID + (GRID - 1)] = (f1 + f2 + f3 + micro) * envelope;
        }
        // ── Step 2: SWE leapfrog — update eta from velocity divergence ─────────
        let mut new_eta = self.eta.clone();
        for z in 1..(GRID - 1) {
            // skip edges — they're driven by Gerstner
            for x in 1..(GRID - 1) {
                let i = z * GRID + x;
                let u_r = self.u[z * GRID + (x + 1)];
                let u_l = self.u[i];
                let v_b = self.v[(z + 1) * GRID + x];
                let v_t = self.v[i];
                new_eta[i] -= DT * H / DX * ((u_r - u_l) + (v_b - v_t));
            }
        }

        // ── Step 3: Update velocities from new eta gradient ────────────────────
        let mut new_u = self.u.clone();
        let mut new_v = self.v.clone();

        for z in 0..GRID {
            for x in 1..GRID {
                let i = z * GRID + x;
                let il = z * GRID + (x - 1);
                new_u[i] -= DT * G / DX * (new_eta[i] - new_eta[il]);
                new_u[i] *= DAMPING;
            }
        }

        for z in 1..GRID {
            for x in 0..GRID {
                let i = z * GRID + x;
                let iu = (z - 1) * GRID + x;
                new_v[i] -= DT * G / DX * (new_eta[i] - new_eta[iu]);
                new_v[i] *= DAMPING;
            }
        }

        self.eta = new_eta;
        self.u = new_u;
        self.v = new_v;
        self.time += DT;
    }

    pub fn grid_size(&self) -> usize {
        GRID
    }

    pub fn get_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::with_capacity(GRID * GRID * 4);
        let half = GRID as f32 / 2.0;
        let mean = self.eta.iter().sum::<f32>() / self.eta.len() as f32;

        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 - half;
                let pz = z as f32 - half;

                // SWE: large-scale energy distribution
                let swe_y = self.eta[z * GRID + x] - mean;

                // Gerstner: sharp surface detail on top
                let (gdx, gdy, gdz) = gerstner_stack(px * WAVE_SCALE, pz * WAVE_SCALE, self.time);

                // SWE energy modulates Gerstner amplitude:
                // high SWE energy → wilder surface detail
                let energy = 1.0 + swe_y.abs() * 1.5;

                vertices.push(px + gdx * energy);
                vertices.push(swe_y + gdy * energy);
                vertices.push(pz + gdz * energy);
                vertices.push(swe_y); // ← color_y: slow SWE only, no flicker
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

    // Interactive splash — adds a Gaussian disturbance on top of the ocean
    // SWE propagates it naturally just like any other energy in the grid
    pub fn splash(&mut self, x: usize, z: usize, amount: f32) {
        let cx = x as f32;
        let cz = z as f32;
        for dz in 1..(GRID - 1) {
            // skip edges — those are Gerstner-driven
            for dx in 1..(GRID - 1) {
                let ddx = dx as f32 - cx;
                let ddz = dz as f32 - cz;
                let r2 = ddx * ddx + ddz * ddz;
                self.eta[dz * GRID + dx] += amount * (-r2 / 8.0).exp();
            }
        }
    }
}
