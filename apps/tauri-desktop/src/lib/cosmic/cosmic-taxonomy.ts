import type { MemoryItem } from "$lib/types/ui";

export type GalaxyDef = {
  id: string;
  label: string;
  tags: string[];
};

export const GALAXY_CATALOG: GalaxyDef[] = [
  { id: "cognition", label: "Cognition", tags: ["nuance", "agent", "insight", "second-brain"] },
  { id: "memoire", label: "Mémoire", tags: ["memoire", "graphe", "backlink", "wikilink", "lancedb"] },
  { id: "technique", label: "Technique", tags: ["rust", "daemon", "websocket", "embedding", "vector"] },
  { id: "interface", label: "Interface", tags: ["cosmic", "godot", "orchestrateur"] },
  { id: "strategie", label: "Stratégie", tags: ["strategie", "trading", "simulation", "ia"] },
  { id: "architecture", label: "Architecture", tags: ["architecture", "cortex", "flux", "neural"] },
  { id: "histoire", label: "Histoire", tags: ["histoire", "chronologie", "1900", "guerre"] },
];

export type CosmicPlacement = {
  memoryId: string;
  galaxyId: string;
  starId: string;
  planetId: string;
};

const PLANET_CAPACITY = 6;

function tagScore(memoryTags: string[], galaxyTags: string[]): number {
  const set = new Set(memoryTags.map((t) => t.toLowerCase()));
  return galaxyTags.reduce((n, t) => n + (set.has(t.toLowerCase()) ? 1 : 0), 0);
}

export function resolveGalaxyId(memory: MemoryItem): string {
  let best = GALAXY_CATALOG[0];
  let bestScore = -1;
  for (const galaxy of GALAXY_CATALOG) {
    const score = tagScore(memory.tags, galaxy.tags);
    if (score > bestScore) {
      bestScore = score;
      best = galaxy;
    }
  }
  if (bestScore <= 0) {
    const fallback = memory.tags[0]?.toLowerCase() ?? "general";
    const match = GALAXY_CATALOG.find((g) => g.tags.includes(fallback));
    return match?.id ?? "architecture";
  }
  return best.id;
}

export function resolveStarId(memory: MemoryItem, galaxyId: string): string {
  const galaxy = GALAXY_CATALOG.find((g) => g.id === galaxyId);
  const galaxyTagSet = new Set((galaxy?.tags ?? []).map((t) => t.toLowerCase()));
  const outsideGalaxy = memory.tags.filter((t) => !galaxyTagSet.has(t.toLowerCase()));
  if (outsideGalaxy.length > 0) {
    return `${galaxyId}::${outsideGalaxy[0].toLowerCase()}`;
  }
  const secondary = memory.tags.find((t) => t.toLowerCase() !== galaxyId);
  if (secondary) return `${galaxyId}::${secondary.toLowerCase()}`;
  const titleToken = memory.title
    .split(/\s+/)
    .map((w) => w.toLowerCase().replace(/[^a-z0-9-]/g, ""))
    .find((w) => w.length > 3 && !galaxyTagSet.has(w));
  if (titleToken) return `${galaxyId}::${titleToken}`;
  return `${galaxyId}::fil-${memory.id.slice(0, 8)}`;
}

export function resolvePlanetId(memory: MemoryItem, starId: string): string {
  const starToken = starId.split("::")[1] ?? "general";
  const other = memory.tags.find((t) => t.toLowerCase() !== starToken);
  const planetKey = other?.toLowerCase() ?? hashSlot(memory.id, PLANET_CAPACITY);
  return `${starId}::${planetKey}`;
}

function hashSlot(id: string, buckets: number): string {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  return `slot-${h % buckets}`;
}

export function assignPlacement(
  memory: MemoryItem,
  cache: Map<string, CosmicPlacement>,
): CosmicPlacement {
  const existing = cache.get(memory.id);
  if (existing) return existing;

  const galaxyId = resolveGalaxyId(memory);
  const starId = resolveStarId(memory, galaxyId);
  const planetId = resolvePlanetId(memory, starId);
  const placement: CosmicPlacement = {
    memoryId: memory.id,
    galaxyId,
    starId,
    planetId,
  };
  cache.set(memory.id, placement);
  return placement;
}

export function galaxyRadiusScale(memoryCount: number, base = 28): number {
  if (memoryCount <= 0) return base * 0.6;
  return base * (0.75 + Math.log10(memoryCount + 1) * 0.55);
}

export function galaxyLabel(galaxyId: string): string {
  return GALAXY_CATALOG.find((g) => g.id === galaxyId)?.label ?? galaxyId;
}

export function starLabel(starId: string): string {
  const token = starId.split("::")[1] ?? "Général";
  return token.charAt(0).toUpperCase() + token.slice(1);
}

export function planetLabel(planetId: string): string {
  const token = planetId.split("::").pop() ?? "Fil";
  if (token.startsWith("slot-")) return "Discussion";
  return token.charAt(0).toUpperCase() + token.slice(1);
}