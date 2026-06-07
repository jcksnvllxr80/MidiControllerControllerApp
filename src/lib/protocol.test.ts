import { describe, it, expect } from "vitest";
import { ENTITY_OPS, PROTOCOL_LABEL, type EntityKind, type Protocol } from "./protocol";

// The wire contract: these op strings must exactly match the Rust `Request`
// enum's snake_case variants in src-tauri/src/protocol/mod.rs. Locked from both
// sides — the Rust test `every_variant_serializes_to_its_locked_op_string`
// asserts the same list.
const WIRE_OPS = [
  "identify",
  "ping",
  "list_sets",
  "get_set",
  "list_songs",
  "get_song",
  "list_pedals",
  "get_pedal",
  "write_set",
  "write_song",
  "write_part",
  "write_pedal",
  "delete_set",
  "delete_song",
  "delete_part",
  "delete_pedal",
  "dpad",
  "short",
] as const;

describe("protocol constants", () => {
  it("has exactly 18 unique wire ops", () => {
    expect(new Set(WIRE_OPS).size).toBe(18);
  });

  it("ENTITY_OPS maps each kind to its CRUD ops", () => {
    expect(ENTITY_OPS.set).toEqual({
      list: "list_sets",
      get: "get_set",
      write: "write_set",
      del: "delete_set",
    });
    expect(ENTITY_OPS.song).toEqual({
      list: "list_songs",
      get: "get_song",
      write: "write_song",
      del: "delete_song",
    });
    expect(ENTITY_OPS.pedal).toEqual({
      list: "list_pedals",
      get: "get_pedal",
      write: "write_pedal",
      del: "delete_pedal",
    });
  });

  it("every ENTITY_OPS op is part of the wire contract", () => {
    const used = Object.values(ENTITY_OPS).flatMap((o) => Object.values(o));
    expect(used.length).toBe(12);
    for (const op of used) {
      expect(WIRE_OPS).toContain(op);
    }
  });

  it("covers exactly the three entity kinds", () => {
    const kinds: EntityKind[] = ["set", "song", "pedal"];
    expect(Object.keys(ENTITY_OPS).sort()).toEqual([...kinds].sort());
  });

  it("PROTOCOL_LABEL has a label for every protocol", () => {
    const protocols: Protocol[] = ["serial", "usb", "wifi", "ethernet", "mock"];
    for (const p of protocols) {
      expect(PROTOCOL_LABEL[p]).toBeTruthy();
    }
    expect(PROTOCOL_LABEL).toEqual({
      serial: "Serial",
      usb: "USB",
      wifi: "Wi-Fi",
      ethernet: "Ethernet",
      mock: "Mock",
    });
  });
});

describe("entity ops integrity", () => {
  it("get/write/delete ops are distinct within each kind", () => {
    for (const k of ["set", "song", "pedal"] as const) {
      const ops = Object.values(ENTITY_OPS[k]);
      expect(new Set(ops).size).toBe(ops.length);
    }
  });

  it("all list ops are unique across kinds", () => {
    const lists = (["set", "song", "pedal"] as const).map((k) => ENTITY_OPS[k].list);
    expect(new Set(lists).size).toBe(3);
  });

  it("PROTOCOL_LABEL has exactly five protocols", () => {
    expect(Object.keys(PROTOCOL_LABEL)).toHaveLength(5);
  });
});
