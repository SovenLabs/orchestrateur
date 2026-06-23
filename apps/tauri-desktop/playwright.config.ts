import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests/visual",
  timeout: 60_000,
  retries: 0,
  use: {
    baseURL: "http://127.0.0.1:1421",
    viewport: { width: 640, height: 480 },
    deviceScaleFactor: 1,
    launchOptions: {
      args: [
        "--use-gl=angle",
        "--use-angle=swiftshader",
        "--enable-unsafe-swiftshader",
        "--ignore-gpu-blocklist",
      ],
    },
  },
  webServer: {
    command: "npx vite --port 1421 --strictPort",
    url: "http://127.0.0.1:1421",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});