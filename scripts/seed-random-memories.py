#!/usr/bin/env python3
"""Génère N mémoires Markdown canoniques pour tests de rendu (graphe / nébuleuse)."""

from __future__ import annotations

import argparse
import random
import secrets
import textwrap
from datetime import UTC, datetime, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MEMORIES_DIR = ROOT / "workspace" / "memories"

PREFIXES = [
    "Analyse", "Note", "Synthèse", "Signal", "Pattern", "Hypothèse", "Trace",
    "Fragment", "Observation", "Modèle", "Corrélation", "Projection", "Réseau",
    "Nœud", "Flux", "Boucle", "Stratégie", "Simulation", "Artefact", "Index",
]
CORES = [
    "neural", "cosmique", "trading", "cortex", "daemon", "nuance", "graphe",
    "embedding", "orchestrateur", "second-brain", "agent", "mémoire", "void",
    "accrétion", "constellation", "insight", "backlink", "wikilink", "phase28",
    "simulation-ia",
]
SUFFIXES = [
    "alpha", "beta", "delta", "sigma", "prime", "v2", "v3", "runtime", "live",
    "batch", "stream", "cluster", "hub", "lane", "pulse", "orbit", "seed",
    "probe", "cache", "vector",
]

TAG_POOL = [
    "architecture", "cortex", "rust", "trading", "simulation", "ia", "second-brain",
    "nuance", "agent", "websocket", "godot", "memoire", "graphe", "embedding",
    "daemon", "orchestrateur", "strategie", "cosmic", "neural", "backlink",
    "wikilink", "insight", "flux", "vector", "lancedb",
]

TOPICS = [
    "corrélation entre activité agent et profondeur de nuance",
    "boucle de rétroaction mémoire → chat → assimilation",
    "simulation de marché appliquée au routing des skills",
    "graphe de backlinks pour navigation insights",
    "embedding sémantique et seuil de similarité",
    "second brain local souverain sans cloud",
    "trou noir UI comme métaphore de densité informationnelle",
    "constellation d'agents synchronisés sur le daemon",
    "nébuleuse mémoire et rendu canvas 2D",
    "tickers de cohérence globale en header",
]


def uuid7_at(offset_ms: int) -> str:
    """UUID v7 déterministe à partir d'un timestamp ms + aléa."""
    unix_ts_ms = offset_ms & 0xFFFFFFFFFFFF
    rand_a = secrets.randbits(12)
    rand_b = secrets.randbits(62)
    uuid_int = (unix_ts_ms << 80) | (0x7 << 76) | (rand_a << 64) | (0x2 << 62) | rand_b
    h = f"{uuid_int:032x}"
    return f"{h[0:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"


def random_title(rng: random.Random) -> str:
    return f"{rng.choice(PREFIXES)} {rng.choice(CORES)} {rng.choice(SUFFIXES)}"


def pick_tags(rng: random.Random, min_count: int = 3) -> list[str]:
    count = rng.randint(min_count, min(6, len(TAG_POOL)))
    return sorted(rng.sample(TAG_POOL, count))


def yaml_quote(s: str) -> str:
    return s.replace('"', '\\"')


def build_memory(
    mem_id: str,
    title: str,
    tags: list[str],
    created: datetime,
    updated: datetime,
    backlinks: list[tuple[str, float, str]],
    body: str,
) -> str:
    bl_lines = []
    for target, score, kind in backlinks:
        bl_lines.append(f"  - target: \"{target}\"")
        bl_lines.append(f"    score: {score:.2f}")
        bl_lines.append(f"    kind: {kind}")
    bl_block = "\n".join(bl_lines) if bl_lines else "  []"
    tags_yaml = ", ".join(f'"{t}"' for t in tags)
    return (
        f"---\n"
        f"id: \"{mem_id}\"\n"
        f"title: \"{yaml_quote(title)}\"\n"
        f"tags: [{tags_yaml}]\n"
        f"created_at: \"{created.strftime('%Y-%m-%dT%H:%M:%SZ')}\"\n"
        f"updated_at: \"{updated.strftime('%Y-%m-%dT%H:%M:%SZ')}\"\n"
        f"backlinks:\n{bl_block}\n"
        f"---\n\n"
        f"{body.strip()}\n"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("-n", "--count", type=int, default=100)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    MEMORIES_DIR.mkdir(parents=True, exist_ok=True)

    base_ms = int(datetime(2026, 6, 1, tzinfo=UTC).timestamp() * 1000)
    ids = [uuid7_at(base_ms + i * 17) for i in range(args.count)]

    created = []
    for i in range(args.count):
        mem_id = ids[i]
        title = random_title(rng)
        tags = pick_tags(rng, min_count=3)

        # ≥3 backlinks vers d'autres mémoires (anneau + saut pseudo-aléatoire)
        targets = {
            ids[(i + 1) % args.count],
            ids[(i + 2) % args.count],
            ids[(i + 3) % args.count],
            ids[(i + 7) % args.count],
        }
        backlinks: list[tuple[str, float, str]] = []
        for j, target in enumerate(sorted(targets)):
            kind = "explicit_wikilink" if j == 0 else "semantic"
            score = 1.0 if kind == "explicit_wikilink" else round(0.76 + rng.random() * 0.22, 2)
            backlinks.append((target, score, kind))

        wikilinks = " ".join(f"[[{t}]]" for t in list(targets)[:3])
        topic = rng.choice(TOPICS)
        body = textwrap.dedent(
            f"""
            # {title}

            Observation seed **#{i + 1}** — {topic}.

            Liens explicites : {wikilinks}

            ## Tags actifs
            {", ".join(f"`{t}`" for t in tags)}

            ## Note
            Mémoire générée pour test de rendu (nébuleuse, nodes orbitaux, insights).
            """
        ).strip()

        ts = datetime(2026, 6, 1, tzinfo=UTC) + timedelta(minutes=i * 13)
        md = build_memory(mem_id, title, tags, ts, ts, backlinks, body)
        path = MEMORIES_DIR / f"{mem_id}.md"
        path.write_text(md, encoding="utf-8")
        created.append(path)

    print(f"OK: {len(created)} mémoires écrites dans {MEMORIES_DIR}")
    print(f"Exemple: {created[0].name}")


if __name__ == "__main__":
    main()