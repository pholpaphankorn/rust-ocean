use crate::waves::{gerstner_height, noise, amplitude_envelope, edge_activity, WAVE_SCALE};

pub const GRID: usize = 128;
const DT: f32 = 0.1;
const G: f32 = 9.8;
const DX: f32 = 1.0;
const H: f32 = 1.0;
const DAMPING: f32 = 0.995;

pub struct FluidGrid {
    pub eta: Vec<f32>,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
}

impl FluidGrid {
    pub fn new() -> Self {
        let size = GRID * GRID;
        Self {
            eta: vec![0.0; size],
            u: vec![0.0; size],
            v: vec![0.0; size],
        }
    }

    pub fn compute_step(&mut self, time: f32) {
        let half = GRID as f32 / 2.0;

        // 1. North edge
        let north_activity = edge_activity(0.0, time);
        for x in 0..GRID {
            let px = (x as f32 - half) * WAVE_SCALE;
            let base = gerstner_height(px, 0.0, time);
            let f1 = (px * 2.3 + time * 0.97).sin() * 0.4;
            let f2 = (px * 5.1 + time * 1.73).sin() * 0.2;
            let f3 = (px * 0.7 + time * 0.41).sin() * 0.6;
            let envelope = amplitude_envelope(x as f32, time);
            let micro = noise(x as f32, time) * 0.15;
            self.eta[x] = (base + f1 + f2 + f3 + micro) * envelope * north_activity;
        }

        // 2. South edge
        let south_activity = edge_activity(3.7, time);
        for x in 0..GRID {
            let px = (x as f32 - half) * WAVE_SCALE;
            let f1 = (px * 3.1 + time * 1.13).sin() * 0.5;
            let f2 = (px * 1.7 + time * 0.67).sin() * 0.3;
            let f3 = (px * 6.3 + time * 2.11).sin() * 0.15;
            let envelope = amplitude_envelope(x as f32 + 50.0, time + 7.3);
            let micro = noise(x as f32 + 100.0, time) * 0.12;
            self.eta[(GRID - 1) * GRID + x] = (f1 + f2 + f3 + micro) * envelope * south_activity;
        }

        // 3. West edge
        let west_activity = edge_activity(7.1, time);
        for z in 0..GRID {
            let pz = (z as f32 - half) * WAVE_SCALE;
            let f1 = (pz * 1.9 + time * 0.83).sin() * 0.45;
            let f2 = (pz * 4.3 + time * 1.51).sin() * 0.18;
            let f3 = (pz * 0.9 + time * 0.37).sin() * 0.5;
            let envelope = amplitude_envelope(z as f32 + 25.0, time + 3.1);
            let micro = noise(z as f32 + 200.0, time) * 0.1;
            self.eta[z * GRID] = (f1 + f2 + f3 + micro) * envelope * west_activity;
        }

        // 4. East edge
        let east_activity = edge_activity(13.3, time);
        for z in 0..GRID {
            let pz = (z as f32 - half) * WAVE_SCALE;
            let f1 = (pz * 2.7 + time * 1.29).sin() * 0.35;
            let f2 = (pz * 0.5 + time * 0.53).sin() * 0.55;
            let f3 = (pz * 7.1 + time * 1.97).sin() * 0.12;
            let envelope = amplitude_envelope(z as f32 + 75.0, time + 11.7);
            let micro = noise(z as f32 + 300.0, time) * 0.1;
            self.eta[z * GRID + (GRID - 1)] = (f1 + f2 + f3 + micro) * envelope * east_activity;
        }

        // SWE Leapfrog: Update eta
        let mut new_eta = self.eta.clone();
        for z in 1..(GRID - 1) {
            for x in 1..(GRID - 1) {
                let i = z * GRID + x;
                let u_r = self.u[z * GRID + (x + 1)];
                let u_l = self.u[i];
                let v_b = self.v[(z + 1) * GRID + x];
                let v_t = self.v[i];
                new_eta[i] -= DT * H / DX * ((u_r - u_l) + (v_b - v_t));
            }
        }

        // Update velocities from gradient
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
    }

    pub fn add_splash(&mut self, x: usize, z: usize, amount: f32) {
        let cx = x as f32;
        let cz = z as f32;
        for dz in 1..(GRID - 1) {
            for dx in 1..(GRID - 1) {
                let ddx = dx as f32 - cx;
                let ddz = dz as f32 - cz;
                let r2 = ddx * ddx + ddz * ddz;
                self.eta[dz * GRID + dx] += amount * (-r2 / 8.0).exp();
            }
        }
    }
}