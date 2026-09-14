import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";

test("the getting-started example runs an uploaded model", async ({ page }) => {
  const markdown = readFileSync(new URL("../../docs/javascript/get-started.md", import.meta.url), "utf8");
  const example = markdown.match(/```typescript\n([\s\S]*?)\n```/)?.[1];
  if (!example) throw new Error("The getting-started example is missing");
  const messages: string[] = [];
  const errors: string[] = [];
  page.on("console", message => {
    messages.push(message.text());
    if (message.type() === "error") errors.push(`${message.location().url}: ${message.text()}`);
  });
  page.on("pageerror", error => errors.push(error.message));

  await page.route("**/favicon.ico", route => route.fulfill({ status: 204 }));
  await page.goto("/");
  await page.addScriptTag({ type: "module", content: example });
  const picker = page.locator('input[type="file"]').last();
  await picker.setInputFiles(fileURLToPath(new URL("../../python/tests/data/rain_subcatch.inp", import.meta.url)));

  await expect.poll(() => messages.some(message => /Binary output: [1-9]\d+ bytes/.test(message))).toBe(true);
  expect(messages.some(message => message.includes("Flow Routing Continuity"))).toBe(true);
  expect(errors).toEqual([]);
});
