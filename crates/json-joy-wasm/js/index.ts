/**
 * json-joy-wasm — TypeScript wrapper for the Rust/WASM json-joy port.
 *
 * ## Quick start
 *
 * ```ts
 * import init, { Model as WasmModel } from '../pkg/json_joy_wasm';
 * import { Model } from 'json-joy-wasm';
 *
 * // Initialise WASM once at app start
 * await init();
 * Model.init(WasmModel);
 *
 * // Create a document
 * const model = Model.create();
 * model.api.set({ title: '', done: false });
 * model.api.str(['title']).ins(0, 'Buy milk');
 *
 * // Flush local changes to a binary patch
 * const patch = model.api.flush();
 *
 * // Sync with a peer
 * const peer = Model.create();
 * peer.applyPatch(patch);
 * console.log(peer.view()); // { title: 'Buy milk', done: false }
 * ```
 *
 * @module
 */

export { Model } from './src/Model';
export type { WasmModelClass } from './src/Model';
export { ModelApi } from './src/ModelApi';
export { Patch } from './src/Patch';
export {
  NodeApi,
  ObjApi,
  StrApi,
  ArrApi,
  BinApi,
  ValApi,
  VecApi,
  ConApi,
} from './src/nodes';
export type { WasmModel } from './src/nodes';
export type { ApiPath, PathKey } from './src/types';
