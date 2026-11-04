// fm, the reference implementation of the specification: `FM_BIN`, else `fm` on PATH. The site's
// examples are checked against it with `fm validate`.

import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import { tempDir } from "./temp.js";

const FM = process.env.FM_BIN?.trim() || "fm";

/** Whether an fm that has `fm validate` runs here. */
export function fmAvailable(): boolean {
  const r = spawnSync(FM, ["validate", "--help"], { encoding: "utf8" });
  return r.status === 0 && r.stdout.includes("AUTOMATION.md");
}

export type FmAnswer =
  | { ok: true; name: string; automation: Record<string, any> }
  | { ok: false; error: string };

/** `text` as fm reads it from `<dir>/AUTOMATION.md`. */
export function fmValidate(text: string, dir: string): FmAnswer {
  const folder = path.join(tempDir("aa-fm-"), dir);
  mkdirSync(folder, { recursive: true });
  writeFileSync(path.join(folder, "AUTOMATION.md"), text);
  const r = spawnSync(FM, ["validate", folder, "--json"], { encoding: "utf8" });
  if (r.error) throw r.error;
  return JSON.parse(r.stdout) as FmAnswer;
}
