/**
 * Génère deflection.dat et inverse_radius.dat (format ebruneton BSD-3).
 * Format : [width, height, ...texel RG float32] — voir preprocess_main.cc
 * Approximation numérique Schwarzschild — remplaçable par `make` vendor.
 */
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dir = dirname(fileURLToPath(import.meta.url));
const OUT = join(__dir, "..", "public", "cosmic", "precomputed");

const DEF_W = 512;
const DEF_H = 512;
const INV_W = 64;
const INV_H = 32;
const kMu = 4 / 27;
const PI = Math.PI;

function getRayDeflectionTextureUFromEsquare(e_square) {
  if (e_square < kMu) {
    return 0.5 - Math.sqrt(-Math.log(1 - e_square / kMu) * (1 / 50));
  }
  return 0.5 + Math.sqrt(-Math.log(1 - kMu / e_square) * (1 / 50));
}

function getUapsisFromEsquare(e_square) {
  const x = (2 / kMu) * e_square - 1;
  return 1 / 3 + (2 / 3) * Math.sin((Math.asin(x) * 1) / 3);
}

function getRayDeflectionTextureVFromEsquareAndU(e_square, u) {
  if (e_square > kMu) {
    const x = u < 2 / 3 ? -Math.sqrt(2 / 3 - u) : Math.sqrt(u - 2 / 3);
    return (Math.sqrt(2 / 3) + x) / (Math.sqrt(2 / 3) + Math.sqrt(1 / 3));
  }
  const ua = getUapsisFromEsquare(e_square);
  return 1 - Math.sqrt(Math.max(1 - u / ua, 0));
}

function inverseDeflectionU(texU) {
  let lo = 0.0001;
  let hi = 2;
  for (let i = 0; i < 48; i++) {
    const mid = (lo + hi) * 0.5;
    if (getRayDeflectionTextureUFromEsquare(mid) < texU) lo = mid;
    else hi = mid;
  }
  return (lo + hi) * 0.5;
}

function inverseDeflectionV(texV, e_square) {
  let lo = 0.0001;
  let hi = 1;
  for (let i = 0; i < 48; i++) {
    const mid = (lo + hi) * 0.5;
    if (getRayDeflectionTextureVFromEsquareAndU(e_square, mid) < texV) lo = mid;
    else hi = mid;
  }
  return (lo + hi) * 0.5;
}

function approxDeflection(e_square, u) {
  if (e_square >= kMu) {
    const turns = Math.max(0, Math.log(e_square / kMu) * 2.5);
    return PI * (1.5 + turns * 0.5) + (u - 2 / 3) * 1.2;
  }
  const ua = getUapsisFromEsquare(e_square);
  const t = u / Math.max(ua, 0.001);
  return PI * 0.5 * t * t + Math.atan(Math.sqrt(e_square / Math.max(kMu - e_square, 1e-6))) * 0.8;
}

function approxDeflectionTime(e_square, u) {
  return Math.log(1 + e_square * 8) * (1 - u) * 0.4 + u * 0.15;
}

function getPhiUbFromEsquare(e_square) {
  return (1 + e_square) / (1 / 3 + 2 * e_square * Math.sqrt(e_square));
}

function getRayInverseRadiusTextureUFromEsquare(e_square) {
  return 1 / (1 + 6 * e_square);
}

function inverseInvRadiusU(texU) {
  let lo = 0.0001;
  let hi = 8;
  for (let i = 0; i < 48; i++) {
    const mid = (lo + hi) * 0.5;
    if (getRayInverseRadiusTextureUFromEsquare(mid) < texU) lo = mid;
    else hi = mid;
  }
  return (lo + hi) * 0.5;
}

function approxInverseRadius(e_square, phi, phiUb) {
  const t = phi / Math.max(phiUb, 0.001);
  const base = e_square * Math.sin(phi * (0.85 + e_square * 0.1));
  return Math.min(1, Math.max(0, base * (1 - t * 0.3) + (1 - t) * 0.05));
}

function approxInvTime(e_square, phi) {
  return phi * (0.2 + e_square * 0.05);
}

function writeDat(path, w, h, fill) {
  const buf = new Float32Array(2 + w * h * 2);
  buf[0] = w;
  buf[1] = h;
  let i = 2;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const texU = (x + 0.5) / w;
      const texV = (y + 0.5) / h;
      const [a, b] = fill(texU, texV);
      buf[i++] = a;
      buf[i++] = b;
    }
  }
  writeFileSync(path, Buffer.from(buf.buffer));
}

mkdirSync(OUT, { recursive: true });

writeDat(join(OUT, "deflection.dat"), DEF_W, DEF_H, (texU, texV) => {
  const e2 = inverseDeflectionU(texU);
  const u = inverseDeflectionV(texV, e2);
  return [approxDeflection(e2, u), approxDeflectionTime(e2, u)];
});

writeDat(join(OUT, "inverse_radius.dat"), INV_W, INV_H, (texU, texV) => {
  const e2 = inverseInvRadiusU(texU);
  const phiUb = getPhiUbFromEsquare(e2);
  const phi = texV * phiUb;
  return [approxInverseRadius(e2, phi, phiUb), approxInvTime(e2, phi)];
});

console.log(`Wrote ${OUT}/deflection.dat (${DEF_W}x${DEF_H})`);
console.log(`Wrote ${OUT}/inverse_radius.dat (${INV_W}x${INV_H})`);