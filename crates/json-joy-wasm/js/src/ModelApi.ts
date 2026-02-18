/**
 * ModelApi — the root editing API for a JSON CRDT document.
 *
 * Mirrors `ModelApi` from `json-joy`, sitting at the root of the document
 * and delegating mutations to the underlying WASM `Model`.
 *
 * Obtain an instance via `model.api`.
 */

import { Patch } from './Patch';
import {
  ArrApi,
  BinApi,
  ConApi,
  NodeApi,
  ObjApi,
  StrApi,
  type WasmModel,
  ValApi,
  VecApi,
} from './nodes';
import type { ApiPath, PathKey } from './types';
import { normalizePath, pathToJson } from './types';

/**
 * Root-level editing API for a JSON CRDT document.
 *
 * `ModelApi` extends {@link NodeApi} so all navigation methods (`obj`, `str`,
 * `arr`, …) work from the document root, exactly as in the upstream library.
 *
 * @example
 * ```ts
 * const model = Model.create();
 * const api = model.api;
 *
 * api.set({ title: '', done: false });
 * api.str(['title']).ins(0, 'Buy milk');
 * const patch = api.flush();
 * ```
 *
 * @category Local API
 */
export class ModelApi extends NodeApi {
  constructor(wasm: WasmModel) {
    // The root ModelApi always starts at the empty path (document root).
    super(wasm, []);
  }

  // ── Set ────────────────────────────────────────────────────────────────────

  /**
   * Replace the entire document value.
   *
   * Mirrors `model.api.set(value)`.  Scalars (null, boolean, number) are
   * stored as constants.  **Strings** are stored as CRDT-editable `str` nodes
   * so that you can call `api.str([key]).ins(...)` immediately after.  Objects
   * and arrays are stored recursively as structural CRDT nodes.
   *
   * ```ts
   * api.set({ name: '', count: 0 });
   * api.str(['name']).ins(0, 'Alice');  // works — name is a StrNode
   * ```
   *
   * To store a string as an immutable constant, use an object wrapper with
   * `api.obj().set({ key: 'const' })` instead.
   */
  set(value: unknown): void {
    this.wasm.apiSet(JSON.stringify(value));
  }

  // ── Typed navigation (override return types for method chaining) ──────────

  /** @inheritdoc */
  override in(sub?: ApiPath): NodeApi {
    return new NodeApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override obj(sub?: ApiPath): ObjApi {
    return new ObjApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override str(sub?: ApiPath): StrApi {
    return new StrApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override arr(sub?: ApiPath): ArrApi {
    return new ArrApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override bin(sub?: ApiPath): BinApi {
    return new BinApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override val(sub?: ApiPath): ValApi {
    return new ValApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override vec(sub?: ApiPath): VecApi {
    return new VecApi(this.wasm, normalizePath(this.path, sub));
  }

  /** @inheritdoc */
  override con(sub?: ApiPath): ConApi {
    return new ConApi(this.wasm, normalizePath(this.path, sub));
  }

  // ── Flush / apply ──────────────────────────────────────────────────────────

  /**
   * Flush all pending local changes into a single binary {@link Patch} and
   * clear the internal change log.
   *
   * Returns an empty patch when there are no pending operations.
   *
   * Mirrors `model.api.flush()`.
   */
  flush(): Patch {
    return new Patch(this.wasm.apiFlush());
  }

  /**
   * Apply pending changes to the in-memory document and discard them without
   * returning a patch.
   *
   * Mirrors `model.api.apply()`.
   */
  apply(): void {
    this.wasm.apiApply();
  }

  // ── View ───────────────────────────────────────────────────────────────────

  /**
   * Return the current JSON view of the whole document.
   */
  override view(): unknown {
    return this.wasm.view();
  }

  // ── Diff ───────────────────────────────────────────────────────────────────

  /**
   * Compute the CRDT patch that transforms this document into `next`, apply it
   * locally, and return the binary patch.
   *
   * Returns an empty patch when the document is already equal to `next`.
   */
  diffApply(next: unknown): Patch {
    return new Patch(this.wasm.diffApply(JSON.stringify(next)));
  }
}
