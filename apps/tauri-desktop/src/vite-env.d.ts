/// <reference types="vite/client" />

declare module "*.glsl?raw" {
  const content: string;
  export default content;
}

interface ImportMetaEnv {
  readonly VITE_ORCHESTRATEUR_WS_URL?: string;
  readonly VITE_ORCHESTRATEUR_DAEMON_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}