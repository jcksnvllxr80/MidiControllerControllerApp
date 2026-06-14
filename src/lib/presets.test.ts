import { describe, it, expect } from "vitest";
import { presetOptions, presetChoices } from "./presets";

// Real "Set Preset" shapes taken from the firmware's data/pedals/*.json.
const BIGSKY = { "Set Preset": { display: { first_page: 0, page_width: 3 }, min: 0, max: 299 } };
const TIMELINE = { "Set Preset": { display: { first_page: 0, page_width: 2 }, min: 0, max: 199 } };
const IRIDIUM = { "Set Preset": { min: 0, max: 299 } };
const QUARTZ = { "Set Preset": { cc: 97, min: 0, max: 127 } };
const SUPERSWITCHER = { "Set Preset": { "program change": { min: 0, max: 127 } } };
const SCARLETT = { "Set Preset": { "control change": { options: ["TS808", "Plexi", "Klone"] } } };

describe("presetOptions", () => {
  it("flat numeric range from top-level min/max (Iridium)", () => {
    const o = presetOptions(IRIDIUM)!;
    expect(o.length).toBe(300);
    expect(o[0]).toEqual({ value: 0, label: "0" });
    expect(o[299]).toEqual({ value: 299, label: "299" });
  });

  it("numeric range under program change (SuperSwitcher)", () => {
    const o = presetOptions(SUPERSWITCHER)!;
    expect(o.length).toBe(128);
    expect(o[5]).toEqual({ value: 5, label: "5" });
  });

  it("flat numeric range with cc present (QuartzV2)", () => {
    const o = presetOptions(QUARTZ)!;
    expect(o.length).toBe(128);
  });

  it("string options under control change (ScarlettLove)", () => {
    const o = presetOptions(SCARLETT)!;
    expect(o.map((x) => x.value)).toEqual(["TS808", "Plexi", "Klone"]);
    expect(o.map((x) => x.label)).toEqual(["TS808", "Plexi", "Klone"]);
  });

  it("display block re-labels the range as page/button banks (BigSky, width 3)", () => {
    const o = presetOptions(BIGSKY)!;
    expect(o.length).toBe(300);
    expect(o[0]).toEqual({ value: 0, label: "Pg0-Btn1" });
    expect(o[1]).toEqual({ value: 1, label: "Pg0-Btn2" });
    expect(o[2]).toEqual({ value: 2, label: "Pg0-Btn3" });
    expect(o[3]).toEqual({ value: 3, label: "Pg1-Btn1" });
    expect(o[299]).toEqual({ value: 299, label: "Pg99-Btn3" });
  });

  it("display block with page_width 2 (TimeLine)", () => {
    const o = presetOptions(TIMELINE)!;
    expect(o[2]).toEqual({ value: 2, label: "Pg1-Btn1" });
    expect(o[5]).toEqual({ value: 5, label: "Pg2-Btn2" });
  });

  it("returns null when there is no Set Preset or no derivable range", () => {
    expect(presetOptions({})).toBeNull();
    expect(presetOptions(null)).toBeNull();
    expect(presetOptions({ "Set Preset": { "control change": {} } })).toBeNull();
  });
});

describe("presetChoices", () => {
  const defs = { BigSky: BIGSKY, ScarlettLove: SCARLETT };

  it("returns null for an unknown / definition-less pedal", () => {
    expect(presetChoices({}, "Nope", 0)).toBeNull();
  });

  it("uses the derived options when the current value is in range", () => {
    const c = presetChoices(defs, "BigSky", 3)!;
    expect(c.length).toBe(300);
    expect(c[3]).toEqual({ value: 3, label: "Pg1-Btn1" });
  });

  it("preserves an out-of-range current value as the first choice", () => {
    const c = presetChoices(defs, "ScarlettLove", "Custom")!;
    expect(c[0]).toEqual({ value: "Custom", label: "Custom (current)" });
    expect(c.slice(1).map((o) => o.value)).toEqual(["TS808", "Plexi", "Klone"]);
  });

  it("does not duplicate a current value already present (number vs string)", () => {
    const c = presetChoices(defs, "BigSky", "3"); // string "3" matches value 3
    expect(c!.length).toBe(300);
  });
});
