// Entry point — wires Rust sim, renderer, camera, and wave generator
// This is the only file that knows about all other modules

import init, { SimState } from '../pkg/ocean_wasm';
import { Renderer } from './renderer';
import { Camera } from './camera';
import { WaveGenerator } from './simulation';

async function main(): Promise<void> {
  // --- WASM ---
  await init();
  const sim: SimState = new SimState();
  const GRID: number = sim.grid_size();
  const SPLASH_AMOUNT: number = 20;

  // --- WEBGPU SETUP ---
  const canvas = document.getElementById('ocean') as HTMLCanvasElement;

  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error('No WebGPU adapter found');

  const device = await adapter.requestDevice();
  const context = canvas.getContext('webgpu');
  if (!context) throw new Error('Could not get WebGPU context');

  const format = navigator.gpu.getPreferredCanvasFormat();
  context.configure({ device, format });

  // --- MODULES ---
  const renderer = new Renderer(canvas, device, format);
  const camera = new Camera(canvas, { maxZoom: GRID * 2 });
  const waves = new WaveGenerator(sim, GRID);

  renderer.uploadIndices(new Uint32Array(sim.get_indices()));

  // --- CLICK TO SPLASH ---
  canvas.addEventListener('click', () => {
    waves.randomSplash(SPLASH_AMOUNT);
  });

  // --- RENDER LOOP ---
  function frame(): void {
    // waves.update(); // generate waves from UI controls
    sim.step(); // advance Rust physics

    const verts = new Float32Array(sim.get_vertices());
    const mvp = camera.getMVP(canvas);

    renderer.draw(context!, verts, mvp);

    requestAnimationFrame(frame);
  }

  frame();
}

main().catch(console.error);
