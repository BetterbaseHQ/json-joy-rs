/**
 * A single key in a path — either an object key (string) or array index (number).
 */
export type PathKey = string | number;

/**
 * A path argument accepted by node navigation methods.
 *
 * Mirrors `ApiPath` from the upstream `json-joy` `nodes.ts`.
 * - `undefined` / `null` → stay at the current node
 * - A single string or number → single-key path
 * - An array → multi-step path
 */
export type ApiPath = PathKey | PathKey[] | undefined | null;

/**
 * Append `sub` to `base`, returning the combined absolute path.
 *
 * **Note**: JSON Pointer strings (those starting with `/`) are **not**
 * supported as path arguments.  Pass an explicit array instead:
 * ```ts
 * // ✗  api.str('/users/0/name')  — treats the whole string as a single key
 * // ✓  api.str(['users', 0, 'name'])
 * ```
 * Strings beginning with `/` will throw to surface this misuse early.
 */
export function normalizePath(base: PathKey[], sub?: ApiPath): PathKey[] {
  if (sub === undefined || sub === null) return base;
  if (typeof sub === 'string') {
    if (sub.startsWith('/')) {
      throw new Error(
        `JSON Pointer syntax ("${sub}") is not supported — pass an array path instead`,
      );
    }
    return [...base, sub];
  }
  if (typeof sub === 'number') return [...base, sub];
  return [...base, ...sub];
}

/**
 * Serialize a path to the JSON string expected by the WASM API.
 *
 * An empty path (document root) serializes to `"null"`.
 */
export function pathToJson(path: PathKey[]): string {
  return path.length === 0 ? 'null' : JSON.stringify(path);
}
