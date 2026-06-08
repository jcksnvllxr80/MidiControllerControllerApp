// Turn a thrown error or rejected command into something a human reads.
// Strips the JS "Error:" prefix and softens the known transport messages into
// plain language with a next step, instead of dumping raw exception text.

const FRIENDLY: [RegExp, string][] = [
  [/not connected/i, "Not connected to a device. Scan and connect first."],
  [
    /no identify response|identify returned no data|bad identify payload/i,
    "The device didn't answer the identify handshake. Is the controller firmware running and speaking the protocol?",
  ],
  [/timed out|timeout/i, "The device timed out. Check the cable and try again."],
  [/failed to open|in use|access is denied|permission denied/i, "Couldn't open the port — it may be in use by another program."],
  [/connection closed|closed before response/i, "The connection dropped. Reconnect to continue."],
  [/pending the firmware|not implemented yet/i, "That isn't available for this device yet."],
  [/invalid json/i, "That isn't valid JSON. Fix the highlighted text and try again."],
];

export function humanizeError(e: unknown): string {
  let msg = e instanceof Error ? e.message : String(e);
  msg = msg.replace(/^Error:\s*/i, "").trim();
  for (const [pattern, friendly] of FRIENDLY) {
    if (pattern.test(msg)) return friendly;
  }
  return msg || "Something went wrong.";
}
