import wasm from "../pkg/iidiff_wasm_bg.wasm";
import init, { diff } from "../pkg/iidiff_wasm.js";
await init(wasm);
export { diff };