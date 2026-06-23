// Derive selectable preset options from a pedal's "Set Preset" definition,
// mirroring the controller's encoding so the editor offers the same choices the
// hardware accepts:
//   • a top-level `min`/`max` is a flat numeric range (e.g. Iridium 0–299);
//   • a `display: { first_page, page_width }` block re-labels that range as
//     page/button banks ("Pg1-Btn2") — BigSky (width 3), TimeLine (width 2);
//   • otherwise the range/options live under `control change` / `program change`
//     as either min/max or a string `options` array (e.g. ScarlettLove amps).

export type PresetOption = { value: number | string; label: string };

function minMaxRange(d: any): number[] {
  if (d && typeof d === "object" && "min" in d && "max" in d) {
    const out: number[] = [];
    for (let i = Number(d.min); i <= Number(d.max); i++) out.push(i);
    return out;
  }
  return [];
}

export function presetRange(sp: any): (number | string)[] {
  if (!sp || typeof sp !== "object") return [];
  if ("min" in sp) return minMaxRange(sp);
  const deeper = sp["control change"] ?? sp["program change"];
  if (!deeper) return [];
  const r = minMaxRange(deeper);
  return r.length ? r : Array.isArray(deeper.options) ? deeper.options : [];
}

export function presetLabels(sp: any, range: (number | string)[]): string[] {
  const disp = sp?.display;
  if (!disp || range.length === 0) return range.map(String);
  const firstPage = Number(disp.first_page);
  const pageWidth = Number(disp.page_width);
  const start = Math.min(...range.map(Number));
  return range.map((p) => {
    const n = Number(p);
    const page = Math.trunc((n - start) / pageWidth) + firstPage;
    const btn = (n % pageWidth) + (1 - start);
    return `Pg${page}-Btn${btn}`;
  });
}

/** Options derived from a pedal definition, or null if it declares no presets. */
export function presetOptions(def: any): PresetOption[] | null {
  const sp = def?.["Set Preset"];
  if (!sp) return null;
  const range = presetRange(sp);
  if (range.length === 0) return null;
  const labels = presetLabels(sp, range);
  return range.map((v, i) => ({ value: v, label: labels[i] }));
}

/**
 * Options for a pedal, preserving the current saved value as a selectable choice
 * even if it falls outside the derived set — so opening an editor never silently
 * rewrites a preset the hardware happens to hold.
 */
// ── Per-part parameter editing ──────────────────────────────────────────────
// A pedal definition's `Parameters` group lists editable per-part settings. Only
// direct leaf entries with a `cc` are addressable by the firmware's setParams
// (ConfigLoader reads part.pedals[*].params; MidiPedal resolves each name against
// the Parameters group's direct children). Nested type-conditional groups and
// pure triggers are not stored. Each leaf is one of:
//   • enum  — a `dict` map; the stored value is the chosen dict KEY string;
//   • on/off — `off`/`on` pair, stored as the string "off"/"on";
//   • range — numeric `min`/`max`, stored as a number in [min,max].
export type ParamSpec =
  | { name: string; kind: "enum"; options: string[] }
  | { name: string; kind: "range"; min: number; max: number };

/** Editable per-part parameter specs derived from a pedal definition, in order. */
export function paramSpecs(def: any): ParamSpec[] {
  const params = def?.["Parameters"];
  if (!params || typeof params !== "object") return [];
  const out: ParamSpec[] = [];
  for (const [name, info] of Object.entries(params) as [string, any][]) {
    if (!info || typeof info !== "object") continue;
    if (info.dict && typeof info.dict === "object") {
      out.push({ name, kind: "enum", options: Object.keys(info.dict) });
    } else if (typeof info.min === "number" && typeof info.max === "number") {
      out.push({ name, kind: "range", min: Number(info.min), max: Number(info.max) });
    } else if ("off" in info && "on" in info) {
      out.push({ name, kind: "enum", options: ["off", "on"] });
    }
    // else: nested type group or pure trigger — not a directly-addressable param.
  }
  return out;
}

/** The default value for a freshly-added parameter (first option / range min). */
export function paramDefault(spec: ParamSpec): number | string {
  return spec.kind === "range" ? spec.min : spec.options[0];
}

export function presetChoices(
  defs: Record<string, any>,
  pedal: string,
  current: unknown,
): PresetOption[] | null {
  const opts = presetOptions(defs[pedal]);
  if (!opts) return null;
  if (current != null && !opts.some((o) => String(o.value) === String(current))) {
    return [{ value: current as number | string, label: `${current} (current)` }, ...opts];
  }
  return opts;
}
