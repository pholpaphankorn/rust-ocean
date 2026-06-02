// Does NOT own SimState — receives it from main.ts

import type { SimState } from '../pkg/ocean_wasm';
export class WaveGenerator {
  private sim: SimState;
  private grid: number;

  constructor(sim: SimState, grid: number) {
    this.sim = sim;
    this.grid = grid;
  }

  // Manual splash — called on canvas click
  randomSplash(amount: number): void {
    const x = Math.floor(Math.random() * this.grid);
    const z = Math.floor(Math.random() * this.grid);
    this.sim.splash(x, z, amount);
  }
}
