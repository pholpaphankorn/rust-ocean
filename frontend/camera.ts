// Camera orbit controls + all matrix math
// Owns: theta, phi, radius, mouse/scroll events

interface CameraOptions {
  maxZoom?: number;
}

export class Camera {
  private theta: number = Math.PI / 4;
  private phi: number = Math.PI / 3;
  private radius: number = 35;
  private maxZoom: number;

  private readonly PHI_MIN = 0.1;
  private readonly PHI_MAX = Math.PI / 2 - 0.05;

  constructor(canvas: HTMLCanvasElement, { maxZoom = 100 }: CameraOptions = {}) {
    this.maxZoom = maxZoom;
    this.registerEvents(canvas);
  }

  private registerEvents(canvas: HTMLCanvasElement): void {
    let isDragging = false;
    let lastX = 0;
    let lastY = 0;

    canvas.addEventListener('mousedown', (e: MouseEvent) => {
      isDragging = true;
      lastX = e.clientX;
      lastY = e.clientY;
    });

    window.addEventListener('mouseup', () => {
      isDragging = false;
    });

    window.addEventListener('mousemove', (e: MouseEvent) => {
      if (!isDragging) return;
      this.theta -= (e.clientX - lastX) * 0.01;
      this.phi = Math.max(
        this.PHI_MIN,
        Math.min(this.PHI_MAX, this.phi - (e.clientY - lastY) * 0.01)
      );
      lastX = e.clientX;
      lastY = e.clientY;
    });

    canvas.addEventListener(
      'wheel',
      (e: WheelEvent) => {
        this.radius = Math.max(10, Math.min(this.maxZoom, this.radius + e.deltaY * 0.05));
        e.preventDefault();
      },
      { passive: false }
    );
  }

  getEye(): [number, number, number] {
    return [
      this.radius * Math.sin(this.phi) * Math.sin(this.theta),
      this.radius * Math.cos(this.phi),
      this.radius * Math.sin(this.phi) * Math.cos(this.theta),
    ];
  }

  getMVP(canvas: HTMLCanvasElement): Float32Array {
    const eye = this.getEye();
    const proj = Camera.perspective(Math.PI / 4, canvas.width / canvas.height, 0.1, 1000);
    const view = Camera.lookAt(eye, [0, 0, 0], [0, 1, 0]);
    return Camera.mat4Mul(proj, view);
  }

  // --- Static matrix math ---

  static mat4Mul(a: Float32Array, b: Float32Array): Float32Array {
    const out = new Float32Array(16);
    for (let col = 0; col < 4; col++)
      for (let row = 0; row < 4; row++) {
        let sum = 0;
        for (let k = 0; k < 4; k++) sum += a[k * 4 + row] * b[col * 4 + k];
        out[col * 4 + row] = sum;
      }
    return out;
  }

  static perspective(fovY: number, aspect: number, near: number, far: number): Float32Array {
    const f = 1.0 / Math.tan(fovY / 2);
    const m = new Float32Array(16);
    m[0] = f / aspect;
    m[5] = f;
    m[10] = far / (near - far);
    m[11] = -1;
    m[14] = (near * far) / (near - far);
    return m;
  }

  static lookAt(
    eye: [number, number, number],
    center: [number, number, number],
    up: [number, number, number]
  ): Float32Array {
    let zx = eye[0] - center[0],
      zy = eye[1] - center[1],
      zz = eye[2] - center[2];
    const zl = Math.hypot(zx, zy, zz);
    zx /= zl;
    zy /= zl;
    zz /= zl;

    let xx = up[1] * zz - up[2] * zy;
    let xy = up[2] * zx - up[0] * zz;
    let xz = up[0] * zy - up[1] * zx;
    const xl = Math.hypot(xx, xy, xz);
    xx /= xl;
    xy /= xl;
    xz /= xl;

    const yx = zy * xz - zz * xy;
    const yy = zz * xx - zx * xz;
    const yz = zx * xy - zy * xx;

    const m = new Float32Array(16);
    m[0] = xx;
    m[1] = yx;
    m[2] = zx;
    m[3] = 0;
    m[4] = xy;
    m[5] = yy;
    m[6] = zy;
    m[7] = 0;
    m[8] = xz;
    m[9] = yz;
    m[10] = zz;
    m[11] = 0;
    m[12] = -(xx * eye[0] + xy * eye[1] + xz * eye[2]);
    m[13] = -(yx * eye[0] + yy * eye[1] + yz * eye[2]);
    m[14] = -(zx * eye[0] + zy * eye[1] + zz * eye[2]);
    m[15] = 1;
    return m;
  }
}
