/**
 * A JSON CRDT patch encoded as binary bytes.
 *
 * Mirrors the upstream `Patch` from `json-joy/json-crdt-patch`.
 * The `bin` property holds the raw bytes that can be sent to peers.
 */
export class Patch {
  constructor(public readonly bin: Uint8Array) {}

  /** True when the patch contains no operations. */
  get isEmpty(): boolean {
    return this.bin.length === 0;
  }
}
