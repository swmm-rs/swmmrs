import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  testMatch: "**/*.spec.ts",
  timeout: 120_000,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:8086",
    headless: true,
    launchOptions: {
      ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
    },
  },
  webServer: [{
    command: "node serve.js",
    url: "http://127.0.0.1:8086",
    env: { PORT: "8086" },
    reuseExistingServer: false,
  }, {
    command: "node serve.js --no-isolation",
    url: "http://127.0.0.1:8087",
    env: { PORT: "8087" },
    reuseExistingServer: false,
  }],
});
