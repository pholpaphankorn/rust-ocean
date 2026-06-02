// WebGPU renderer
// Owns: device, pipeline, buffers, depth texture, draw calls
// Knows nothing about camera math or wave logic

import { oceanShader } from './shaders';

export class Renderer {
  private device: GPUDevice;
  private canvas: HTMLCanvasElement;
  private pipeline: GPURenderPipeline;
  private depthTexture: GPUTexture;
  private uniformBuffer: GPUBuffer;
  private bindGroup: GPUBindGroup;
  private indexBuffer: GPUBuffer | null = null;
  private indexCount: number = 0;

  constructor(canvas: HTMLCanvasElement, device: GPUDevice, format: GPUTextureFormat) {
    this.canvas = canvas;
    this.device = device;

    // --- SHADER ---
    const shader = device.createShaderModule({ code: oceanShader });

    // --- PIPELINE ---
    this.pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shader,
        entryPoint: 'vs_main',
        buffers: [
          {
            arrayStride: 16, // ← 4 floats × 4 bytes
            attributes: [
              { shaderLocation: 0, offset: 0, format: 'float32x3' }, // xyz
              { shaderLocation: 1, offset: 12, format: 'float32' }, // color_y
            ],
          },
        ],
      },
      fragment: {
        module: shader,
        entryPoint: 'fs_main',
        targets: [{ format }],
      },
      primitive: { topology: 'triangle-list', cullMode: 'none' },
      depthStencil: {
        format: 'depth24plus',
        depthWriteEnabled: true,
        depthCompare: 'less',
      },
    });

    // --- DEPTH TEXTURE ---
    this.depthTexture = device.createTexture({
      size: [canvas.width, canvas.height],
      format: 'depth24plus',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });

    // --- UNIFORM BUFFER (MVP matrix) ---
    this.uniformBuffer = device.createBuffer({
      size: 64, // 4×4 float32
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    this.bindGroup = device.createBindGroup({
      layout: this.pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: this.uniformBuffer } }],
    });
  }

  // Call once after sim is ready — indices never change
  uploadIndices(indexData: Uint32Array): void {
    this.indexCount = indexData.length;
    this.indexBuffer = this.device.createBuffer({
      size: indexData.byteLength,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(this.indexBuffer, 0, indexData);
  }

  // Call every frame
  draw(context: GPUCanvasContext, verts: Float32Array, mvp: Float32Array): void {
    if (!this.indexBuffer) throw new Error('Index buffer not uploaded');

    // Upload MVP + vertices
    this.device.queue.writeBuffer(this.uniformBuffer, 0, mvp);

    const vertBuffer = this.device.createBuffer({
      size: verts.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(vertBuffer, 0, verts);

    // Render pass
    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          clearValue: { r: 0.05, g: 0.1, b: 0.2, a: 1 },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
      depthStencilAttachment: {
        view: this.depthTexture.createView(),
        depthClearValue: 1.0,
        depthLoadOp: 'clear',
        depthStoreOp: 'store',
      },
    });

    pass.setPipeline(this.pipeline);
    pass.setBindGroup(0, this.bindGroup);
    pass.setVertexBuffer(0, vertBuffer);
    pass.setIndexBuffer(this.indexBuffer, 'uint32');
    pass.drawIndexed(this.indexCount);
    pass.end();

    this.device.queue.submit([encoder.finish()]);
  }
}
