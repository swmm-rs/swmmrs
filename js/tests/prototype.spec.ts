import { expect, test } from "@playwright/test";

test("parallel routing, reruns, external files, snapshots, reports and cleanup", async ({ page }) => {
  await page.goto("/test.html");
  await page.waitForFunction(() => document.documentElement.dataset.result !== undefined);
  expect(await page.locator("#result").textContent()).toMatch(/^PASS:/);
});
