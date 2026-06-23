import type { RenderPresetConfig } from "$lib/cosmic/render-preset";

export type CosmicExportOptions = {
  durationSec?: number;
  fps?: number;
  mimeType?: string;
  width?: number;
  height?: number;
  videoBitsPerSecond?: number;
  fixedTime?: number;
  preset?: RenderPresetConfig;
};

export async function recordCosmicLoop(
  canvas: HTMLCanvasElement,
  options: CosmicExportOptions = {},
): Promise<Blob> {
  const preset = options.preset;
  const durationSec = options.durationSec ?? 5;
  const fps = options.fps ?? preset?.exportMax.fps ?? 30;
  const mimeType = options.mimeType ?? pickWebmMime();
  const bitrate = options.videoBitsPerSecond ?? preset?.exportMax.bitrate ?? 8_000_000;

  if (!mimeType || typeof canvas.captureStream !== "function") {
    throw new Error("Enregistrement vidéo non supporté dans cet environnement");
  }

  if (options.width && options.height) {
    canvas.width = options.width;
    canvas.height = options.height;
  }

  const stream = canvas.captureStream(fps);
  const recorder = new MediaRecorder(stream, { mimeType, videoBitsPerSecond: bitrate });
  const chunks: Blob[] = [];

  return new Promise((resolve, reject) => {
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data);
    };
    recorder.onerror = () => reject(new Error("Échec MediaRecorder"));
    recorder.onstop = () => {
      resolve(new Blob(chunks, { type: mimeType }));
    };

    recorder.start(200);
    window.setTimeout(() => {
      if (recorder.state !== "inactive") recorder.stop();
    }, durationSec * 1000);
  });
}

export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function pickWebmMime(): string | null {
  const candidates = [
    "video/webm;codecs=vp9",
    "video/webm;codecs=vp8",
    "video/webm",
  ];
  for (const type of candidates) {
    if (MediaRecorder.isTypeSupported(type)) return type;
  }
  return null;
}