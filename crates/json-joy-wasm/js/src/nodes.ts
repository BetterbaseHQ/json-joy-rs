/**
 * Node-level editing API classes.
 *
 * Each class holds an **absolute path** from the document root and delegates
 * every mutation to the underlying WASM `Model` instance.  Navigation methods
 * (`obj`, `str`, `arr`, …) return new API objects with the extended path,
 * enabling fluent chaining:
 *
 * ```ts
 * model.api.obj(['users', 0]).str('name').ins(0, 'Alice');
 * ```
 *
 * Mirrors the upstream `nodes.ts` API from `json-joy`.
 */

import type { ApiPath, PathKey } from './types';
import { normalizePath, pathToJson } from './types';

// ---------------------------------------------------------------------------
// WASM model interface
// We import only the type here so the source compiles before wasm-pack runs.
// The concrete class is injected at runtime through the `WasmModel` parameter.
// ---------------------------------------------------------------------------

/**
 * Full interface of the Rust-generated WASM `Model` instance.
 *
 * This mirrors the `#[wasm_bindgen] impl Model` public API in `lib.rs`.
 * Method names match the `js_name` attributes in the Rust source.
 */
export interface WasmModel {
  // ── Lifecycle ──────────────────────────────────────────────────────────────
  /** Binary structural encoding of this document. */
  toBinary(): Uint8Array;
  /** Current JSON view of the whole document. */
  view(): unknown;
  /** The session ID of the local logical clock (as BigInt). */
  sid(): bigint;
  /** Fork this document with an optional new session ID. */
  fork(sid?: bigint): WasmModel;
  /** Apply a remote patch (binary) to this document. */
  applyPatch(bytes: Uint8Array): void;

  // ── Editing ────────────────────────────────────────────────────────────────
  apiSet(json_str: string): void;
  apiObjSet(path_json: string, entries_json: string): void;
  apiObjDel(path_json: string, keys_json: string): void;
  apiVecSet(path_json: string, entries_json: string): void;
  apiValSet(path_json: string, value_json: string): void;
  apiNewStr(obj_path_json: string, key: string, initial_text: string): void;
  apiStrIns(path_json: string, index: number, text: string): void;
  apiStrDel(path_json: string, index: number, length: number): void;
  apiBinIns(path_json: string, index: number, data: Uint8Array): void;
  apiBinDel(path_json: string, index: number, length: number): void;
  apiArrIns(path_json: string, index: number, values_json: string): void;
  apiArrUpd(path_json: string, index: number, value_json: string): void;
  apiArrDel(path_json: string, index: number, length: number): void;

  // ── Flush ──────────────────────────────────────────────────────────────────
  apiFlush(): Uint8Array;
  apiApply(): void;

  // ── Length queries ─────────────────────────────────────────────────────────
  apiStrLen(path_json: string): number;
  apiArrLen(path_json: string): number;
  apiBinLen(path_json: string): number;
  apiVecLen(path_json: string): number;

  // ── View helpers ───────────────────────────────────────────────────────────
  viewAt(path_json: string): unknown;

  // ── Diff ───────────────────────────────────────────────────────────────────
  diffApply(next_json_str: string): Uint8Array;
}

// ---------------------------------------------------------------------------
// NodeApi — base class shared by all node wrappers
// ---------------------------------------------------------------------------

/**
 * Generic CRDT node API.  Provides navigation to typed child APIs and a
 * `view()` convenience method.
 *
 * @category Local API
 */
export class NodeApi {
  constructor(
    protected readonly wasm: WasmModel,
    protected readonly path: PathKey[],
  ) {}

  // ── Navigation ─────────────────────────────────────────────────────────────

  /** Navigate to a child and return a generic {@link NodeApi}. */
  in(sub?: ApiPath): NodeApi {
    return new NodeApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to an object child and return an {@link ObjApi}. */
  obj(sub?: ApiPath): ObjApi {
    return new ObjApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to a string child and return a {@link StrApi}. */
  str(sub?: ApiPath): StrApi {
    return new StrApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to an array child and return an {@link ArrApi}. */
  arr(sub?: ApiPath): ArrApi {
    return new ArrApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to a binary blob child and return a {@link BinApi}. */
  bin(sub?: ApiPath): BinApi {
    return new BinApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to a `val` (LWW register) child and return a {@link ValApi}. */
  val(sub?: ApiPath): ValApi {
    return new ValApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to a `vec` (tuple) child and return a {@link VecApi}. */
  vec(sub?: ApiPath): VecApi {
    return new VecApi(this.wasm, normalizePath(this.path, sub));
  }

  /** Navigate to a constant (`con`) child and return a {@link ConApi}. */
  con(sub?: ApiPath): ConApi {
    return new ConApi(this.wasm, normalizePath(this.path, sub));
  }

  // ── Read ───────────────────────────────────────────────────────────────────

  /**
   * Return the current JSON view of this node (does not cross to JS until you
   * call this method).
   */
  view(): unknown {
    return this.wasm.viewAt(pathToJson(this.path));
  }
}

// ---------------------------------------------------------------------------
// ConApi
// ---------------------------------------------------------------------------

/**
 * Read-only API for a `con` constant node.
 *
 * @category Local API
 */
export class ConApi extends NodeApi {}

// ---------------------------------------------------------------------------
// ValApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for a `val` (Last-Write-Wins register) node.
 *
 * @category Local API
 */
export class ValApi extends NodeApi {
  /**
   * Replace the value held by this register.
   *
   * Scalar values (null/boolean/number/string) are stored as constants.
   * Objects and arrays are stored as structural CRDT nodes.
   */
  set(value: unknown): void {
    this.wasm.apiValSet(pathToJson(this.path), JSON.stringify(value));
  }

  /**
   * Return a generic {@link NodeApi} for the inner node of this register.
   */
  get(): NodeApi {
    return new NodeApi(this.wasm, this.path);
  }
}

// ---------------------------------------------------------------------------
// ObjApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for an `obj` (map / object) node.
 *
 * @category Local API
 */
export class ObjApi extends NodeApi {
  /**
   * Set one or more key→value pairs on this object.
   *
   * Scalar values (null/boolean/number/string) are stored as constants.
   * Objects and arrays are stored as structural CRDT nodes.
   * To store a **collaboratively-editable** string, use {@link newStr} instead.
   */
  set(entries: Record<string, unknown>): void {
    this.wasm.apiObjSet(pathToJson(this.path), JSON.stringify(entries));
  }

  /**
   * Delete keys from this object.
   */
  del(keys: string[]): void {
    this.wasm.apiObjDel(pathToJson(this.path), JSON.stringify(keys));
  }

  /**
   * Create a new CRDT-editable string (`str` node) at `key` on this object,
   * optionally seeding it with `initial`.
   *
   * This is the equivalent of using `s.str(initial)` in an upstream schema.
   * Strings added via `set({key: ''})` are immutable constants; use this
   * method when you need character-level collaborative editing.
   *
   * ```ts
   * model.api.obj().newStr('title', 'Untitled');
   * model.api.str(['title']).ins(0, 'My Doc ');
   * ```
   */
  newStr(key: string, initial = ''): void {
    this.wasm.apiNewStr(pathToJson(this.path), key, initial);
  }

  /**
   * Return `true` if this object currently has an entry for `key`.
   *
   * Note: the view is materialized to check existence.  For hot paths,
   * prefer reading `view()` once and checking the result.
   */
  has(key: string): boolean {
    const v = this.wasm.viewAt(pathToJson(this.path));
    return typeof v === 'object' && v !== null && key in (v as Record<string, unknown>);
  }
}

// ---------------------------------------------------------------------------
// StrApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for a `str` (CRDT string / RGA) node.
 *
 * All positions are **character indices** in the current (materialized) string.
 *
 * @category Local API
 */
export class StrApi extends NodeApi {
  /**
   * Insert `text` at character position `index`.
   *
   * @param index 0-based insert position (characters, not bytes).
   * @param text  Text to insert.
   */
  ins(index: number, text: string): void {
    this.wasm.apiStrIns(pathToJson(this.path), index, text);
  }

  /**
   * Delete `length` characters starting at `index`.
   *
   * @param index  0-based start position.
   * @param length Number of characters to remove.
   */
  del(index: number, length: number): void {
    this.wasm.apiStrDel(pathToJson(this.path), index, length);
  }

  /**
   * Return the current character length of this string without materializing
   * its full value.
   */
  length(): number {
    return this.wasm.apiStrLen(pathToJson(this.path));
  }
}

// ---------------------------------------------------------------------------
// BinApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for a `bin` (binary / byte-array RGA) node.
 *
 * @category Local API
 */
export class BinApi extends NodeApi {
  /**
   * Insert `data` bytes at byte position `index`.
   */
  ins(index: number, data: Uint8Array): void {
    this.wasm.apiBinIns(pathToJson(this.path), index, data);
  }

  /**
   * Delete `length` bytes starting at `index`.
   */
  del(index: number, length: number): void {
    this.wasm.apiBinDel(pathToJson(this.path), index, length);
  }

  /**
   * Return the current byte length without materializing the full blob.
   */
  length(): number {
    return this.wasm.apiBinLen(pathToJson(this.path));
  }
}

// ---------------------------------------------------------------------------
// ArrApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for an `arr` (array RGA) node.
 *
 * @category Local API
 */
export class ArrApi extends NodeApi {
  /**
   * Insert `values` at position `index`.
   *
   * Strings are stored as CRDT-editable `str` nodes (matching upstream
   * behaviour).  Use nested arrays/objects to store structural nodes.
   *
   * @param index  0-based insert position.
   * @param values Elements to insert.
   */
  ins(index: number, values: unknown[]): void {
    this.wasm.apiArrIns(pathToJson(this.path), index, JSON.stringify(values));
  }

  /**
   * Append `values` at the end of the array.
   */
  push(...values: unknown[]): void {
    this.ins(this.length(), values);
  }

  /**
   * Overwrite the element at `index` with a new value.
   *
   * Mirrors upstream `ArrApi.upd(index, value)`.
   *
   * @param index 0-based position of the element to replace.
   * @param value New value.
   */
  upd(index: number, value: unknown): void {
    this.wasm.apiArrUpd(pathToJson(this.path), index, JSON.stringify(value));
  }

  /**
   * Delete `length` elements starting at `index`.
   */
  del(index: number, length: number): void {
    this.wasm.apiArrDel(pathToJson(this.path), index, length);
  }

  /**
   * Return the current element count without materializing the array.
   */
  length(): number {
    return this.wasm.apiArrLen(pathToJson(this.path));
  }
}

// ---------------------------------------------------------------------------
// VecApi
// ---------------------------------------------------------------------------

/**
 * Local changes API for a `vec` (fixed-size tuple) node.
 *
 * @category Local API
 */
export class VecApi extends NodeApi {
  /**
   * Set indexed entries.
   *
   * @param entries List of `[index, value]` pairs.
   */
  set(entries: [index: number, value: unknown][]): void {
    this.wasm.apiVecSet(pathToJson(this.path), JSON.stringify(entries));
  }

  /**
   * Append values starting at the current end of the tuple.
   *
   * Mirrors upstream `VecApi.push(...values)`.
   */
  push(...values: unknown[]): void {
    const length = this.length();
    this.set(values.map((value, index) => [length + index, value]));
  }

  /**
   * Return the number of slots in this tuple.
   */
  length(): number {
    return this.wasm.apiVecLen(pathToJson(this.path));
  }
}
