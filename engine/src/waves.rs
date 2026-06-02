// Scale factor: maps grid coords → wave space
pub const WAVE_SCALE: f32 = 0.08;

pub struct GerstnerWave {
    pub amplitude: f32,
    pub frequency: f32,
    pub speed: f32,
    pub dir_x: f32,
    pub dir_z: f32,
    pub steepness: f32,
}

impl GerstnerWave {
    pub fn displace(&self, x: f32, z: f32, time: f32) -> (f32, f32, f32) {
        let dot = self.dir_x * x + self.dir_z * z;
        let phase = self.frequency * dot + self.speed * time;

        let dy = self.amplitude * phase.sin();
        let dx = self.steepness * self.amplitude * self.dir_x * phase.cos();
        let dz = self.steepness * self.amplitude * self.dir_z * phase.cos();

        (dx, dy, dz)
    }
}

pub fn gerstner_stack(x: f32, z: f32, time: f32) -> (f32, f32, f32) {
    let waves: [GerstnerWave; 4] = [
        GerstnerWave {
            amplitude: 0.8,
            frequency: 0.15,
            speed: 1.2,
            dir_x: 1.0,
            dir_z: 0.0,
            steepness: 0.5,
        },
        GerstnerWave {
            amplitude: 0.35,
            frequency: 0.3,
            speed: 1.8,
            dir_x: 0.7,
            dir_z: 0.7,
            steepness: 0.4,
        },
        GerstnerWave {
            amplitude: 0.18,
            frequency: 0.55,
            speed: 2.4,
            dir_x: 0.2,
            dir_z: 0.98,
            steepness: 0.3,
        },
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

pub fn gerstner_height(x: f32, z: f32, time: f32) -> f32 {
    let (_, dy, _) = gerstner_stack(x, z, time);
    dy
}

pub fn noise(x: f32, t: f32) -> f32 {
    let n = (x * 127.1 + t * 311.7).sin() * 43758.545;
    n - n.floor() - 0.5
}

pub fn amplitude_envelope(x: f32, t: f32) -> f32 {
    let slow = (x * 0.3 + t * 0.07).sin();
    let drift = (x * 0.11 + t * 0.03).cos();
    0.3 + (slow + drift + 2.0) * 0.35
}

pub fn edge_activity(seed: f32, t: f32) -> f32 {
    let a = (t * 0.031 + seed).sin();
    let b = (t * 0.017 + seed * 2.7).sin();
    let raw = (a + b) * 0.5;
    let t01 = (raw + 1.0) * 0.5;
    t01 * t01 * (3.0 - 2.0 * t01)
}
