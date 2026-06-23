export type Vec2 = { x: number; y: number };

export type ParticleKind = "feed" | "ambient";

export type Particle = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  hue: number;
  size: number;
  kind: ParticleKind;
  trail: number;
};

const MAX_PARTICLES = 240;

export function feedBurstHue(depth: number): number {
  return 255 + depth * 75;
}

export function createParticleSystem() {
  const particles: Particle[] = [];

  function emitFeedBurst(from: Vec2, to: Vec2, count = 24, depth = 0.5): void {
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const hueBase = feedBurstHue(depth);
    for (let i = 0; i < count && particles.length < MAX_PARTICLES; i++) {
      const spread = 6 + depth * 6;
      particles.push({
        x: from.x + (Math.random() - 0.5) * spread,
        y: from.y + (Math.random() - 0.5) * spread,
        vx: dx * (0.014 + depth * 0.008) + (Math.random() - 0.5) * 0.5,
        vy: dy * (0.014 + depth * 0.008) + (Math.random() - 0.5) * 0.5,
        life: 1,
        maxLife: 0.7 + Math.random() * 0.55,
        hue: hueBase + Math.random() * 35,
        size: 1.4 + Math.random() * 2.2 + depth,
        kind: "feed",
        trail: 0.35 + depth * 0.25,
      });
    }
  }

  function emitAmbient(count: number, cx: number, cy: number, radius: number, depth = 0.3): void {
    for (let i = 0; i < count && particles.length < MAX_PARTICLES; i++) {
      const angle = Math.random() * Math.PI * 2;
      const r = radius * (0.55 + Math.random() * 0.55);
      particles.push({
        x: cx + Math.cos(angle) * r,
        y: cy + Math.sin(angle) * r,
        vx: (Math.random() - 0.5) * 0.12,
        vy: (Math.random() - 0.5) * 0.12,
        life: 1,
        maxLife: 2 + Math.random() * 2.5,
        hue: 195 + depth * 70 + Math.random() * 40,
        size: 0.9 + Math.random() * 1.4,
        kind: "ambient",
        trail: 0,
      });
    }
  }

  function step(dt: number, attract: Vec2, strength = 0.02): void {
    for (let i = particles.length - 1; i >= 0; i--) {
      const p = particles[i];
      const pull = p.kind === "feed" ? strength * 1.35 : strength * 0.75;
      const ax = (attract.x - p.x) * pull;
      const ay = (attract.y - p.y) * pull;
      p.vx += ax * dt;
      p.vy += ay * dt;
      p.vx *= 0.985;
      p.vy *= 0.985;
      p.x += p.vx * dt * 60;
      p.y += p.vy * dt * 60;
      p.life -= dt / p.maxLife;
      if (p.life <= 0) particles.splice(i, 1);
    }
  }

  function draw(ctx: CanvasRenderingContext2D): void {
    for (const p of particles) {
      const alpha = Math.max(0, p.life) * (p.kind === "feed" ? 0.9 : 0.72);
      if (p.trail > 0) {
        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(p.x - p.vx * p.trail * 8, p.y - p.vy * p.trail * 8);
        ctx.strokeStyle = `hsla(${p.hue}, 48%, 76%, ${alpha * 0.45})`;
        ctx.lineWidth = p.size * 0.55;
        ctx.stroke();
      }
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, ${p.kind === "feed" ? 52 : 42}%, ${p.kind === "feed" ? 80 : 74}%, ${alpha})`;
      ctx.fill();
    }
  }

  function clear(): void {
    particles.length = 0;
  }

  return { particles, emitFeedBurst, emitAmbient, step, draw, clear };
}

export type ParticleSystem = ReturnType<typeof createParticleSystem>;