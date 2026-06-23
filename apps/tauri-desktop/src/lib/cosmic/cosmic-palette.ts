/**
 * Palette partagée WebGL ↔ Godot (territoire-graphique).
 * Source Godot : starfield_background.gdshader, core_plasma.gdshader
 */
export const COSMIC_PALETTE = {
  void: { hex: "#0a0a12", rgb: [10 / 255, 10 / 255, 18 / 255] as const },
  starfieldTop: [0.02, 0.02, 0.06] as const,
  starfieldHorizon: [0.01, 0.01, 0.02] as const,
  starTint: [0.55, 0.7, 1.0] as const,
  starDensity: 0.35,
  coreColorA: [0.8, 1.0, 1.0] as const,
  coreColorB: [0.67, 0.87, 1.0] as const,
  rimCyan: [0.2, 0.95, 1.0] as const,
  mauve: [200 / 255, 168 / 255, 208 / 255] as const,
  sky: [160 / 255, 196 / 255, 224 / 255] as const,
  diskHot: [1.0, 0.28, 0.04] as const,
  diskCold: [0.38, 0.68, 1.0] as const,
  photonRing: [1.0, 0.96, 0.88] as const,
  /** Couleurs par kind mémoire (Pulse → cosmique). */
  kind: {
    decision: { hex: "#5eead4", rgb: [94 / 255, 234 / 255, 212 / 255] as const },
    dead_end: { hex: "#f87171", rgb: [248 / 255, 113 / 255, 113 / 255] as const },
    pattern: { hex: "#4ade80", rgb: [74 / 255, 222 / 255, 128 / 255] as const },
    context: { hex: "#a0c4e0", rgb: [160 / 255, 196 / 255, 224 / 255] as const },
    progress: { hex: "#fdba74", rgb: [253 / 255, 186 / 255, 116 / 255] as const },
    business: { hex: "#c8a8d0", rgb: [200 / 255, 168 / 255, 208 / 255] as const },
  },
} as const;

export type MemoryKindId =
  | "decision"
  | "dead_end"
  | "pattern"
  | "context"
  | "progress"
  | "business";

export const MEMORY_KIND_LABELS: Record<MemoryKindId, string> = {
  decision: "Décision",
  dead_end: "Impasse",
  pattern: "Pattern",
  context: "Contexte",
  progress: "Progrès",
  business: "Business",
};

export function kindColor(kind: string): string {
  const key = kind as MemoryKindId;
  return COSMIC_PALETTE.kind[key]?.hex ?? COSMIC_PALETTE.kind.context.hex;
}

export type RgbTriplet = readonly [number, number, number];

export function paletteVec3(key: keyof typeof COSMIC_PALETTE): RgbTriplet {
  const v = COSMIC_PALETTE[key];
  if (Array.isArray(v) && v.length === 3 && typeof v[0] === "number") {
    return v as RgbTriplet;
  }
  return COSMIC_PALETTE.mauve;
}