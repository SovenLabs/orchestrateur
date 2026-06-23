import * as THREE from "three";
import type { CosmicCameraState } from "$lib/cosmic/cosmic-camera";
import {
  buildCelestialBodies,
  type BodyLayoutInput,
  type CelestialBody,
} from "$lib/cosmic/three/cosmic-body-layout";

export type PlanetLayer = {
  readonly available: boolean;
  resize: (width: number, height: number, dpr: number) => void;
  render: (input: BodyLayoutInput & { camera: CosmicCameraState; cx: number; cy: number }) => void;
  clear: () => void;
  destroy: () => void;
  getCanvas: () => HTMLCanvasElement;
};

const SPHERE_SEGMENTS = 24;

function kindScale(kind: CelestialBody["kind"]): number {
  if (kind === "star") return 1.0;
  if (kind === "planet") return 1.0;
  return 0.85;
}

export function createCosmicPlanetLayer(canvas: HTMLCanvasElement): PlanetLayer {
  let renderer: THREE.WebGLRenderer | null = null;
  let scene: THREE.Scene | null = null;
  let camera: THREE.OrthographicCamera | null = null;
  let rootGroup: THREE.Group | null = null;
  let camGroup: THREE.Group | null = null;
  let keyLight: THREE.DirectionalLight | null = null;
  let fillLight: THREE.DirectionalLight | null = null;

  const meshPool = new Map<string, THREE.Mesh>();
  let cssWidth = 1;
  let cssHeight = 1;

  try {
    renderer = new THREE.WebGLRenderer({
      canvas,
      alpha: true,
      antialias: true,
      preserveDrawingBuffer: true,
      powerPreference: "high-performance",
    });
    renderer.setClearColor(0x000000, 0);
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.05;

    scene = new THREE.Scene();
    rootGroup = new THREE.Group();
    camGroup = new THREE.Group();
    rootGroup.add(camGroup);
    scene.add(rootGroup);

    camera = new THREE.OrthographicCamera(0, 1, 0, 1, 0.1, 2000);
    camera.position.set(0, 0, 500);
    camera.lookAt(0, 0, 0);

    scene.add(new THREE.AmbientLight(0x334466, 0.35));
    keyLight = new THREE.DirectionalLight(0xffeedd, 1.1);
    keyLight.position.set(-0.6, 0.8, 1.2);
    scene.add(keyLight);
    fillLight = new THREE.DirectionalLight(0x88aacc, 0.45);
    fillLight.position.set(0.7, -0.4, 0.8);
    scene.add(fillLight);
  } catch (e) {
    console.error("[cosmic-planet-layer] init failed", e);
    return {
      available: false,
      resize: () => {},
      render: () => {},
      clear: () => {},
      destroy: () => {},
      getCanvas: () => canvas,
    };
  }

  const r = renderer!;
  const sc = scene!;
  const cam = camera!;
  const root = rootGroup!;
  const camGrp = camGroup!;

  function ensureMesh(body: CelestialBody): THREE.Mesh {
    let mesh = meshPool.get(body.id);
    if (!mesh) {
      const geo = new THREE.SphereGeometry(1, SPHERE_SEGMENTS, SPHERE_SEGMENTS);
      const mat = new THREE.MeshStandardMaterial({
        color: new THREE.Color(body.color[0], body.color[1], body.color[2]),
        emissive: new THREE.Color(body.color[0] * 0.35, body.color[1] * 0.35, body.color[2] * 0.35),
        emissiveIntensity: body.kind === "star" ? 1.2 : 0.55,
        metalness: 0.15,
        roughness: body.kind === "moon" ? 0.85 : 0.55,
      });
      mesh = new THREE.Mesh(geo, mat);
      meshPool.set(body.id, mesh);
      camGrp.add(mesh);
    }

    const mat = mesh.material as THREE.MeshStandardMaterial;
    mat.color.setRGB(body.color[0], body.color[1], body.color[2]);
    mat.emissive.setRGB(body.color[0] * 0.4, body.color[1] * 0.4, body.color[2] * 0.4);
    mat.emissiveIntensity = body.kind === "star" ? 1.4 : body.kind === "moon" ? 0.35 : 0.65;

    const scale = body.radius * kindScale(body.kind);
    mesh.position.set(body.x, -body.y, body.kind === "star" ? 0 : body.kind === "planet" ? -2 : -4);
    mesh.scale.setScalar(scale);
    mesh.visible = true;
    return mesh;
  }

  function syncMeshes(bodies: CelestialBody[]) {
    const active = new Set(bodies.map((b) => b.id));
    for (const body of bodies) ensureMesh(body);
    for (const [id, mesh] of meshPool) {
      if (!active.has(id)) mesh.visible = false;
    }
  }

  return {
    available: true,
    getCanvas: () => canvas,

    resize(width: number, height: number, dpr: number) {
      cssWidth = width;
      cssHeight = height;
      r.setPixelRatio(dpr);
      r.setSize(width, height, false);
      cam.left = 0;
      cam.right = width;
      cam.top = 0;
      cam.bottom = height;
      cam.updateProjectionMatrix();
    },

    render(input) {
      const bodies = buildCelestialBodies(input);
      if (bodies.length === 0) {
        r.clear();
        return;
      }

      syncMeshes(bodies);

      const { camera: camState, cx, cy } = input;
      root.position.set(cx + camState.panX, -(cy + camState.panY), 0);
      root.scale.set(camState.zoom, camState.zoom, 1);

      r.render(sc, cam);
    },

    clear() {
      r.clear();
      for (const mesh of meshPool.values()) mesh.visible = false;
    },

    destroy() {
      for (const mesh of meshPool.values()) {
        mesh.geometry.dispose();
        (mesh.material as THREE.Material).dispose();
        camGrp.remove(mesh);
      }
      meshPool.clear();
      r.dispose();
    },
  };
}