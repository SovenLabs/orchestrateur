import { test, expect } from "@playwright/test";
import { readFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";

const __dir = dirname(fileURLToPath(import.meta.url));
const BASELINES = join(__dir, "baselines");
const DIFF_DIR = join(__dir, "diffs");

function loadPng(path: string): PNG {
  return PNG.sync.read(readFileSync(path));
}

function comparePng(actual: PNG, expected: PNG, diffPath: string): number {
  const { width, height } = actual;
  const diff = new PNG({ width, height });
  const mismatched = pixelmatch(
    actual.data,
    expected.data,
    diff.data,
    width,
    height,
    { threshold: 0.12, includeAA: true },
  );
  mkdirSync(dirname(diffPath), { recursive: true });
  writeFileSync(diffPath, PNG.sync.write(diff));
  return mismatched / (width * height);
}

test.describe("cosmic golden frames", () => {
  test("cinema expanded", async ({ page }) => {
    page.on("pageerror", (err) => console.error("[harness]", err.message));
    await page.goto("/cosmic-harness.html?preset=cinema&time=12.5&dock=0");
    await page.waitForFunction(() => document.body.dataset.ready === "1", null, {
      timeout: 30_000,
    });
    const err = await page.evaluate(() => document.body.dataset.error);
    expect(err, "harness should render WebGL").toBeUndefined();
    await page.waitForTimeout(900);

    const shot = await page.locator("#gl").screenshot();
    const actual = PNG.sync.read(shot);
    const baselinePath = join(BASELINES, "bh-cinema-expanded.png");

    if (!existsSync(baselinePath)) {
      mkdirSync(BASELINES, { recursive: true });
      writeFileSync(baselinePath, shot);
      test.info().attach("baseline-created", { body: shot, contentType: "image/png" });
      return;
    }

    const expected = loadPng(baselinePath);
    const ratio = comparePng(actual, expected, join(DIFF_DIR, "bh-cinema-expanded.png"));
    expect(ratio).toBeLessThan(0.02);
  });

  test("eco docked", async ({ page }) => {
    page.on("pageerror", (err) => console.error("[harness]", err.message));
    await page.goto("/cosmic-harness.html?preset=eco&time=8&dock=1");
    await page.waitForFunction(() => document.body.dataset.ready === "1", null, {
      timeout: 30_000,
    });
    const err = await page.evaluate(() => document.body.dataset.error);
    expect(err, "harness should render WebGL").toBeUndefined();
    await page.waitForTimeout(900);

    const shot = await page.locator("#gl").screenshot();
    const actual = PNG.sync.read(shot);
    const baselinePath = join(BASELINES, "bh-eco-docked.png");

    if (!existsSync(baselinePath)) {
      mkdirSync(BASELINES, { recursive: true });
      writeFileSync(baselinePath, shot);
      return;
    }

    const expected = loadPng(baselinePath);
    const ratio = comparePng(actual, expected, join(DIFF_DIR, "bh-eco-docked.png"));
    expect(ratio).toBeLessThan(0.02);
  });
});