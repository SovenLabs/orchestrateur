import { linkProgram } from "$lib/cosmic/webgl/gl-utils";
import {
  galaxyPosition,
  type CosmicGalaxy,
  type CosmicScene,
} from "$lib/cosmic/cosmic-model";

import vertSrc from "../shaders/galaxy-instanced.vert.glsl?raw";
import fragSrc from "../shaders/galaxy-instanced.frag.glsl?raw";

const GALAXY_TINTS: Record<string, [number, number, number]> = {
  cognition: [200 / 255, 168 / 255, 208 / 255],
  memoire: [160 / 255, 196 / 255, 224 / 255],
  technique: [148 / 255, 200 / 255, 212 / 255],
  interface: [216 / 255, 184 / 255, 168 / 255],
  strategie: [192 / 255, 160 / 255, 200 / 255],
  architecture: [176 / 255, 208 / 255, 192 / 255],
  histoire: [208 / 255, 192 / 255, 160 / 255],
};

export type GalaxyInstanceInput = {
  scene: CosmicScene;
  cx: number;
  cy: number;
  baseRadius: number;
  time: number;
  dockT: number;
  visibility: number;
};

export type GalaxyInstancedLayer = {
  program: WebGLProgram;
  vao: WebGLVertexArrayObject;
  instanceBuffer: WebGLBuffer;
  quadBuffer: WebGLBuffer;
  maxInstances: number;
  locs: {
    resolution: WebGLUniformLocation | null;
    time: WebGLUniformLocation | null;
    bhCenter: WebGLUniformLocation | null;
    cameraPan: WebGLUniformLocation | null;
    cameraZoom: WebGLUniformLocation | null;
  };
  updateInstances: (input: GalaxyInstanceInput) => number;
  draw: (count: number, uniforms: {
    resolution: [number, number];
    time: number;
    bhCenter: [number, number];
    cameraPan: [number, number];
    cameraZoom: number;
  }) => void;
  destroy: () => void;
};

function tint(galaxyId: string): [number, number, number] {
  return GALAXY_TINTS[galaxyId] ?? [170 / 255, 190 / 255, 220 / 255];
}

export function createGalaxyInstancedLayer(
  gl: WebGL2RenderingContext,
  maxInstances = 32,
): GalaxyInstancedLayer | null {
  const program = linkProgram(gl, vertSrc, fragSrc);
  if (!program) return null;

  const quadVerts = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
  const quadBuffer = gl.createBuffer();
  if (!quadBuffer) return null;
  gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, quadVerts, gl.STATIC_DRAW);

  const instanceBuffer = gl.createBuffer();
  if (!instanceBuffer) return null;

  const vao = gl.createVertexArray();
  if (!vao) return null;
  gl.bindVertexArray(vao);

  gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

  gl.bindBuffer(gl.ARRAY_BUFFER, instanceBuffer);
  const stride = 7 * 4;
  gl.enableVertexAttribArray(1);
  gl.vertexAttribPointer(1, 2, gl.FLOAT, false, stride, 0);
  gl.vertexAttribDivisor(1, 1);
  gl.enableVertexAttribArray(2);
  gl.vertexAttribPointer(2, 1, gl.FLOAT, false, stride, 8);
  gl.vertexAttribDivisor(2, 1);
  gl.enableVertexAttribArray(3);
  gl.vertexAttribPointer(3, 3, gl.FLOAT, false, stride, 12);
  gl.vertexAttribDivisor(3, 1);
  gl.enableVertexAttribArray(4);
  gl.vertexAttribPointer(4, 1, gl.FLOAT, false, stride, 24);
  gl.vertexAttribDivisor(4, 1);

  gl.bindVertexArray(null);
  gl.bindBuffer(gl.ARRAY_BUFFER, null);

  const locs = {
    resolution: gl.getUniformLocation(program, "u_resolution"),
    time: gl.getUniformLocation(program, "u_time"),
    bhCenter: gl.getUniformLocation(program, "u_bh_center"),
    cameraPan: gl.getUniformLocation(program, "u_camera_pan"),
    cameraZoom: gl.getUniformLocation(program, "u_camera_zoom"),
  };

  const instanceData = new Float32Array(maxInstances * 7);

  function buildInstances(input: GalaxyInstanceInput): number {
    const visibility = Math.max(0, input.visibility);
    if (visibility < 0.08) return 0;

    let count = 0;
    for (const galaxy of input.scene.galaxies) {
      if (count >= maxInstances) break;
      const pos = galaxyPosition(
        input.cx,
        input.cy,
        input.baseRadius,
        galaxy,
        input.time,
        input.dockT,
      );
      const c = tint(galaxy.id);
      const r = galaxy.isNebula ? galaxy.radius * 0.35 : galaxy.radius * 0.75;
      const i = count * 7;
      instanceData[i] = pos.x;
      instanceData[i + 1] = pos.y;
      instanceData[i + 2] = r * 2.4 * visibility;
      instanceData[i + 3] = c[0];
      instanceData[i + 4] = c[1];
      instanceData[i + 5] = c[2];
      instanceData[i + 6] = galaxy.isNebula ? 0.55 : 1.0;
      count++;
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, instanceBuffer);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, instanceData.subarray(0, count * 7));
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
    return count;
  }

  return {
    program,
    vao,
    instanceBuffer,
    quadBuffer,
    maxInstances,
    locs,
    updateInstances: buildInstances,
    draw(count, uniforms) {
      if (count < 1) return;
      gl.useProgram(program);
      gl.uniform2f(locs.resolution, uniforms.resolution[0], uniforms.resolution[1]);
      gl.uniform1f(locs.time, uniforms.time);
      gl.uniform2f(locs.bhCenter, uniforms.bhCenter[0], uniforms.bhCenter[1]);
      gl.uniform2f(locs.cameraPan, uniforms.cameraPan[0], uniforms.cameraPan[1]);
      gl.uniform1f(locs.cameraZoom, uniforms.cameraZoom);
      gl.bindVertexArray(vao);
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
      gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
      gl.disable(gl.BLEND);
      gl.bindVertexArray(null);
    },
    destroy() {
      gl.deleteProgram(program);
      gl.deleteBuffer(quadBuffer);
      gl.deleteBuffer(instanceBuffer);
      gl.deleteVertexArray(vao);
    },
  };
}

export function galaxyVisibility(dockT: number): number {
  return Math.max(0.1, 1 - dockT * 0.9);
}