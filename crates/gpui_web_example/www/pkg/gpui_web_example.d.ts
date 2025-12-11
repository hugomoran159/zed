/* tslint:disable */
/* eslint-disable */

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______: (a: number, b: number) => void;
  readonly wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___wasm_bindgen_795bb83e9b24f476___JsValue_____: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__wasm_bindgen_795bb83e9b24f476___JsValue____Output_______: (a: number, b: number) => void;
  readonly wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke______: (a: number, b: number) => void;
  readonly __wbindgen_malloc_command_export: (a: number, b: number) => number;
  readonly __wbindgen_realloc_command_export: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store_command_export: (a: number) => void;
  readonly __externref_table_alloc_command_export: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free_command_export: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
