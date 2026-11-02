/** The agent-automations skill stays host-neutral. */

import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { mentionsPi } from "./helpers/pi-only.js";

const ROOT = path.resolve(import.meta.dirname, "..");
const PORTABLE = path.join(ROOT, "skills", "agent-automations");

function walk(dir: string): string[] {
  return readdirSync(dir)
    .sort()
    .flatMap((name) => {
      const full = path.join(dir, name);
      return statSync(full).isDirectory() ? walk(full) : [full];
    });
}

const rel = (file: string) => path.relative(ROOT, file).split(path.sep).join("/");

describe("the agent-automations skill", () => {
  it("knows what counts as a pi-automations mention", () => {
    for (const text of ["run `pa check x`", "pa note", "/automations arm x", "pi-automations", "`pi /login`", "NOT_AN_EDGE"]) {
      expect(mentionsPi(text), text).toBe(true);
    }
    for (const text of ["pass the path", "pages", "`~/.agents/automations/<name>/`", "https://pi.dev", "api", "spa-like"]) {
      expect(mentionsPi(text), text).toBe(false);
    }
  });

  it.each(walk(PORTABLE).map((f) => ({ name: rel(f), file: f })))("$name never names pa, pi or pi-automations", ({ file }) => {
    const lines = readFileSync(file, "utf8").split("\n");
    const found = lines.flatMap((line, i) => (mentionsPi(line) ? [`${i + 1}: ${line}`] : []));
    expect(found).toEqual([]);
  });
});
