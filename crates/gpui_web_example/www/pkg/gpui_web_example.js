let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc_command_export();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

let cachedInt16ArrayMemory0 = null;
function getInt16ArrayMemory0() {
    if (cachedInt16ArrayMemory0 === null || cachedInt16ArrayMemory0.byteLength === 0) {
        cachedInt16ArrayMemory0 = new Int16Array(wasm.memory.buffer);
    }
    return cachedInt16ArrayMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

let cachedInt8ArrayMemory0 = null;
function getInt8ArrayMemory0() {
    if (cachedInt8ArrayMemory0 === null || cachedInt8ArrayMemory0.byteLength === 0) {
        cachedInt8ArrayMemory0 = new Int8Array(wasm.memory.buffer);
    }
    return cachedInt8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint16ArrayMemory0 = null;
function getUint16ArrayMemory0() {
    if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
        cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16ArrayMemory0;
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store_command_export(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

let WASM_VECTOR_LEN = 0;

function wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____(arg0, arg1, arg2) {
    wasm.wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____(arg0, arg1, arg2);
}

function wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___wasm_bindgen_795bb83e9b24f476___JsValue_____(arg0, arg1, arg2) {
    wasm.wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___wasm_bindgen_795bb83e9b24f476___JsValue_____(arg0, arg1, arg2);
}

function wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke______(arg0, arg1) {
    wasm.wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke______(arg0, arg1);
}

const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];

const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];

const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];

const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];

const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];

const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];

const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];

const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];

const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];

const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];

const __wbindgen_enum_GpuLoadOp = ["load", "clear"];

const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];

const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];

const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];

const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];

const __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];

const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];

const __wbindgen_enum_GpuStoreOp = ["store", "discard"];

const __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];

const __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];

const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];

const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];

const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];

const __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];

const __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];

export function main() {
    wasm.main();
}

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Window_2b9b35492d4b2d63 = function(arg0) {
        const ret = arg0.Window;
        return ret;
    };
    imports.wbg.__wbg_WorkerGlobalScope_b4fb13f0ba6527ab = function(arg0) {
        const ret = arg0.WorkerGlobalScope;
        return ret;
    };
    imports.wbg.__wbg___wbindgen_boolean_get_dea25b33882b895b = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? v : undefined;
        return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
    };
    imports.wbg.__wbg___wbindgen_debug_string_adfb662ae34724b6 = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_is_function_8d400b8b1af978cd = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbg___wbindgen_is_null_dfda7d66506c95b5 = function(arg0) {
        const ret = arg0 === null;
        return ret;
    };
    imports.wbg.__wbg___wbindgen_is_undefined_f6b95eab589e0269 = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbg___wbindgen_number_get_9619185a74197f95 = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbg___wbindgen_string_get_a2a31e16edf96e42 = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg__wbg_cb_unref_87dfb5aaa0cbcea7 = function(arg0) {
        arg0._wbg_cb_unref();
    };
    imports.wbg.__wbg_activeTexture_1db0722f00c3f843 = function(arg0, arg1) {
        arg0.activeTexture(arg1 >>> 0);
    };
    imports.wbg.__wbg_activeTexture_59810c16ea8d6e34 = function(arg0, arg1) {
        arg0.activeTexture(arg1 >>> 0);
    };
    imports.wbg.__wbg_addEventListener_6a82629b3d430a48 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_altKey_56d1d642f3a28c92 = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_altKey_e13fae92dfebca3e = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_appendChild_7465eba84213c75f = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.appendChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_attachShader_bc2b53790fd12d3a = function(arg0, arg1, arg2) {
        arg0.attachShader(arg1, arg2);
    };
    imports.wbg.__wbg_attachShader_ce575704294db9cc = function(arg0, arg1, arg2) {
        arg0.attachShader(arg1, arg2);
    };
    imports.wbg.__wbg_beginQuery_71fca84d19c65fb1 = function(arg0, arg1, arg2) {
        arg0.beginQuery(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_beginRenderPass_f36cfdd5825e0c2e = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.beginRenderPass(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_bindAttribLocation_2bf0ba75dbebbc07 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_bindAttribLocation_4e8be7470dd8dd5a = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_bindBufferRange_b775673f1d6f510c = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.bindBufferRange(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
    };
    imports.wbg.__wbg_bindBuffer_110b128c65a97376 = function(arg0, arg1, arg2) {
        arg0.bindBuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindBuffer_c24c31cbec41cb21 = function(arg0, arg1, arg2) {
        arg0.bindBuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindFramebuffer_302dbc9f62d8321e = function(arg0, arg1, arg2) {
        arg0.bindFramebuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindFramebuffer_33b64eb9f536d2b2 = function(arg0, arg1, arg2) {
        arg0.bindFramebuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindRenderbuffer_032b12b73a396d8c = function(arg0, arg1, arg2) {
        arg0.bindRenderbuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindRenderbuffer_43c98d43540f75ae = function(arg0, arg1, arg2) {
        arg0.bindRenderbuffer(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindSampler_b835d52aec542c4c = function(arg0, arg1, arg2) {
        arg0.bindSampler(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindTexture_4537240b278f1d53 = function(arg0, arg1, arg2) {
        arg0.bindTexture(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindTexture_6ed714c0afe8b8d1 = function(arg0, arg1, arg2) {
        arg0.bindTexture(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_bindVertexArrayOES_fdb7e747e386f55a = function(arg0, arg1) {
        arg0.bindVertexArrayOES(arg1);
    };
    imports.wbg.__wbg_bindVertexArray_ced27387a0718508 = function(arg0, arg1) {
        arg0.bindVertexArray(arg1);
    };
    imports.wbg.__wbg_blendColor_e45c66bf83bef98c = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.blendColor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_blendColor_f4107640d80916d6 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.blendColor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_blendEquationSeparate_403e2a62d6e0d67f = function(arg0, arg1, arg2) {
        arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_blendEquationSeparate_e1eb0d0f32ef91af = function(arg0, arg1, arg2) {
        arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_blendEquation_493973ecbb09fe8c = function(arg0, arg1) {
        arg0.blendEquation(arg1 >>> 0);
    };
    imports.wbg.__wbg_blendEquation_e3d6a981d832c9ff = function(arg0, arg1) {
        arg0.blendEquation(arg1 >>> 0);
    };
    imports.wbg.__wbg_blendFuncSeparate_4cca29476893cc61 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    };
    imports.wbg.__wbg_blendFuncSeparate_e5a1bacf4a0700cd = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    };
    imports.wbg.__wbg_blendFunc_046483861de36edd = function(arg0, arg1, arg2) {
        arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_blendFunc_5eed6dc03a180da2 = function(arg0, arg1, arg2) {
        arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_blitFramebuffer_02db7e02b81bd174 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        arg0.blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
    };
    imports.wbg.__wbg_body_544738f8b03aef13 = function(arg0) {
        const ret = arg0.body;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_bufferData_69dbeea8e1d79f7b = function(arg0, arg1, arg2, arg3) {
        arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferData_ac5c7900b06f1517 = function(arg0, arg1, arg2, arg3) {
        arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferData_c75947f383ca8992 = function(arg0, arg1, arg2, arg3) {
        arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferData_cd7c1cdb1eb72df8 = function(arg0, arg1, arg2, arg3) {
        arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferSubData_16db9d7d9f1c86bb = function(arg0, arg1, arg2, arg3) {
        arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_bufferSubData_e256855a0fda09a5 = function(arg0, arg1, arg2, arg3) {
        arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_buffer_6cb2fecb1f253d71 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_button_a54acd25bab5d442 = function(arg0) {
        const ret = arg0.button;
        return ret;
    };
    imports.wbg.__wbg_call_abb4ff46ce38be40 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_changedTouches_002fee0da4489efe = function(arg0) {
        const ret = arg0.changedTouches;
        return ret;
    };
    imports.wbg.__wbg_clearBufferfv_d33b9af84c129287 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.clearBufferfv(arg1 >>> 0, arg2, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_clearBufferiv_ba2da32ddbdf9e20 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_clearBufferuiv_e465a763e54627c1 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_clearDepth_0f3bb08f167cf1f0 = function(arg0, arg1) {
        arg0.clearDepth(arg1);
    };
    imports.wbg.__wbg_clearDepth_49b8cc204e46a1ff = function(arg0, arg1) {
        arg0.clearDepth(arg1);
    };
    imports.wbg.__wbg_clearStencil_159eaeffc88e2487 = function(arg0, arg1) {
        arg0.clearStencil(arg1);
    };
    imports.wbg.__wbg_clearStencil_f1d7134551355df7 = function(arg0, arg1) {
        arg0.clearStencil(arg1);
    };
    imports.wbg.__wbg_clear_00ac71df5db8ab17 = function(arg0, arg1) {
        arg0.clear(arg1 >>> 0);
    };
    imports.wbg.__wbg_clear_52caf9271911674b = function(arg0, arg1) {
        arg0.clear(arg1 >>> 0);
    };
    imports.wbg.__wbg_clientWaitSync_42970d3aaa2e5351 = function(arg0, arg1, arg2, arg3) {
        const ret = arg0.clientWaitSync(arg1, arg2 >>> 0, arg3 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_clientX_2ea1e898842f21ed = function(arg0) {
        const ret = arg0.clientX;
        return ret;
    };
    imports.wbg.__wbg_clientY_d12086a5a6a3a9d5 = function(arg0) {
        const ret = arg0.clientY;
        return ret;
    };
    imports.wbg.__wbg_clipboard_c210ce30f20907dd = function(arg0) {
        const ret = arg0.clipboard;
        return ret;
    };
    imports.wbg.__wbg_colorMask_27d9f83dd2189ed6 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
    };
    imports.wbg.__wbg_colorMask_f000b510fac0bd7c = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
    };
    imports.wbg.__wbg_compileShader_ac0bf6f0837881c3 = function(arg0, arg1) {
        arg0.compileShader(arg1);
    };
    imports.wbg.__wbg_compileShader_ba337110bed419e1 = function(arg0, arg1) {
        arg0.compileShader(arg1);
    };
    imports.wbg.__wbg_compressedTexSubImage2D_4ab2b43cacd95564 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
    };
    imports.wbg.__wbg_compressedTexSubImage2D_bc669b55bfad0d12 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
    };
    imports.wbg.__wbg_compressedTexSubImage2D_c9d1d1f196bce860 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
    };
    imports.wbg.__wbg_compressedTexSubImage3D_376b9e3cdbccd59b = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10);
    };
    imports.wbg.__wbg_compressedTexSubImage3D_bbc0d31582e3a014 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
    };
    imports.wbg.__wbg_configure_ad5aa321838c8e3b = function() { return handleError(function (arg0, arg1) {
        arg0.configure(arg1);
    }, arguments) };
    imports.wbg.__wbg_confirm_b165cbd0f4493563 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.confirm(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_contentRect_1806147dfdc380d8 = function(arg0) {
        const ret = arg0.contentRect;
        return ret;
    };
    imports.wbg.__wbg_copyBufferSubData_74ad55c13c5b2ae2 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
    };
    imports.wbg.__wbg_copyTexSubImage2D_593b8653753bc7d3 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    };
    imports.wbg.__wbg_copyTexSubImage2D_7f4e6e26c0eff156 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    };
    imports.wbg.__wbg_copyTexSubImage3D_c66982c639aa21c4 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
    };
    imports.wbg.__wbg_createBindGroupLayout_b87a1f26ed22bd5d = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.createBindGroupLayout(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createBindGroup_dfdadbbcf4dcae54 = function(arg0, arg1) {
        const ret = arg0.createBindGroup(arg1);
        return ret;
    };
    imports.wbg.__wbg_createBuffer_465b645a46535184 = function(arg0) {
        const ret = arg0.createBuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createBuffer_8601b8ec330ab49d = function(arg0) {
        const ret = arg0.createBuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createBuffer_fb1752eab5cb2a7f = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.createBuffer(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createCommandEncoder_92b1c283a0372974 = function(arg0, arg1) {
        const ret = arg0.createCommandEncoder(arg1);
        return ret;
    };
    imports.wbg.__wbg_createElement_da4ed2b219560fc6 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createFramebuffer_5d000a6cde602c77 = function(arg0) {
        const ret = arg0.createFramebuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createFramebuffer_934b44643ffd067a = function(arg0) {
        const ret = arg0.createFramebuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createPipelineLayout_c97169a1a177450e = function(arg0, arg1) {
        const ret = arg0.createPipelineLayout(arg1);
        return ret;
    };
    imports.wbg.__wbg_createProgram_023ba0fc6ff6efd6 = function(arg0) {
        const ret = arg0.createProgram();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createProgram_ffe9d4a2cba210f4 = function(arg0) {
        const ret = arg0.createProgram();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createQuery_427027f57b8d51cc = function(arg0) {
        const ret = arg0.createQuery();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createRenderPipeline_ab453ccc40539bc0 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.createRenderPipeline(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createRenderbuffer_6b6220d1a07652a9 = function(arg0) {
        const ret = arg0.createRenderbuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createRenderbuffer_f869ce6d85370a7a = function(arg0) {
        const ret = arg0.createRenderbuffer();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createSampler_4c0a0f10a4d901b3 = function(arg0) {
        const ret = arg0.createSampler();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createSampler_fdf4c92b3a0a4810 = function(arg0, arg1) {
        const ret = arg0.createSampler(arg1);
        return ret;
    };
    imports.wbg.__wbg_createShaderModule_159013272c1b4c4c = function(arg0, arg1) {
        const ret = arg0.createShaderModule(arg1);
        return ret;
    };
    imports.wbg.__wbg_createShader_4626088b63c33727 = function(arg0, arg1) {
        const ret = arg0.createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createShader_f88f9b82748ef6c0 = function(arg0, arg1) {
        const ret = arg0.createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createTexture_092a9cf5106b1805 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.createTexture(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createTexture_41211a4e8ae0afec = function(arg0) {
        const ret = arg0.createTexture();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createTexture_4d5934eb9772b5fe = function(arg0) {
        const ret = arg0.createTexture();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createVertexArrayOES_7bcc20082143e8f2 = function(arg0) {
        const ret = arg0.createVertexArrayOES();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createVertexArray_997b3c5b1091afd9 = function(arg0) {
        const ret = arg0.createVertexArray();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_createView_e743725c577bafe5 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.createView(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_ctrlKey_487597b9069da036 = function(arg0) {
        const ret = arg0.ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_b391e5105c3f6e76 = function(arg0) {
        const ret = arg0.ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_cullFace_767c25333fcc7c8b = function(arg0, arg1) {
        arg0.cullFace(arg1 >>> 0);
    };
    imports.wbg.__wbg_cullFace_88f07a3436967138 = function(arg0, arg1) {
        arg0.cullFace(arg1 >>> 0);
    };
    imports.wbg.__wbg_data_0ce3dd82dc5c9488 = function(arg0, arg1) {
        const ret = arg1.data;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_data_ba1e638a3b5a1da7 = function(arg0, arg1) {
        const ret = arg1.data;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_deleteBuffer_5ed1698208181e1f = function(arg0, arg1) {
        arg0.deleteBuffer(arg1);
    };
    imports.wbg.__wbg_deleteBuffer_ba7f1164cc23b2ca = function(arg0, arg1) {
        arg0.deleteBuffer(arg1);
    };
    imports.wbg.__wbg_deleteFramebuffer_71a99ec4adbfc3f2 = function(arg0, arg1) {
        arg0.deleteFramebuffer(arg1);
    };
    imports.wbg.__wbg_deleteFramebuffer_d25c0dc61ce8eda7 = function(arg0, arg1) {
        arg0.deleteFramebuffer(arg1);
    };
    imports.wbg.__wbg_deleteProgram_3bf297a31d0e6e48 = function(arg0, arg1) {
        arg0.deleteProgram(arg1);
    };
    imports.wbg.__wbg_deleteProgram_62774baacb13ff2b = function(arg0, arg1) {
        arg0.deleteProgram(arg1);
    };
    imports.wbg.__wbg_deleteQuery_9ae103bb04e9a99d = function(arg0, arg1) {
        arg0.deleteQuery(arg1);
    };
    imports.wbg.__wbg_deleteRenderbuffer_3e536cf09d672302 = function(arg0, arg1) {
        arg0.deleteRenderbuffer(arg1);
    };
    imports.wbg.__wbg_deleteRenderbuffer_ada437284f7fb4f2 = function(arg0, arg1) {
        arg0.deleteRenderbuffer(arg1);
    };
    imports.wbg.__wbg_deleteSampler_993727fa1d567ed5 = function(arg0, arg1) {
        arg0.deleteSampler(arg1);
    };
    imports.wbg.__wbg_deleteShader_c357bb8fbede8370 = function(arg0, arg1) {
        arg0.deleteShader(arg1);
    };
    imports.wbg.__wbg_deleteShader_c686dd351de5a068 = function(arg0, arg1) {
        arg0.deleteShader(arg1);
    };
    imports.wbg.__wbg_deleteSync_f5db5552febb6818 = function(arg0, arg1) {
        arg0.deleteSync(arg1);
    };
    imports.wbg.__wbg_deleteTexture_2a9b703dc2df5657 = function(arg0, arg1) {
        arg0.deleteTexture(arg1);
    };
    imports.wbg.__wbg_deleteTexture_875f8d84e74610a0 = function(arg0, arg1) {
        arg0.deleteTexture(arg1);
    };
    imports.wbg.__wbg_deleteVertexArrayOES_c17582be9fb07775 = function(arg0, arg1) {
        arg0.deleteVertexArrayOES(arg1);
    };
    imports.wbg.__wbg_deleteVertexArray_af80f68f0bea25b7 = function(arg0, arg1) {
        arg0.deleteVertexArray(arg1);
    };
    imports.wbg.__wbg_deltaMode_d74ec093e23ffeec = function(arg0) {
        const ret = arg0.deltaMode;
        return ret;
    };
    imports.wbg.__wbg_deltaX_41f7678c94b10355 = function(arg0) {
        const ret = arg0.deltaX;
        return ret;
    };
    imports.wbg.__wbg_deltaY_3f10fd796fae2a0f = function(arg0) {
        const ret = arg0.deltaY;
        return ret;
    };
    imports.wbg.__wbg_depthFunc_30cd9028f7f0cb4e = function(arg0, arg1) {
        arg0.depthFunc(arg1 >>> 0);
    };
    imports.wbg.__wbg_depthFunc_eb0c2c825938bb33 = function(arg0, arg1) {
        arg0.depthFunc(arg1 >>> 0);
    };
    imports.wbg.__wbg_depthMask_317f5412242ac5d5 = function(arg0, arg1) {
        arg0.depthMask(arg1 !== 0);
    };
    imports.wbg.__wbg_depthMask_eabc1830c04e8fca = function(arg0, arg1) {
        arg0.depthMask(arg1 !== 0);
    };
    imports.wbg.__wbg_depthRange_599ac7ebc9b76a2c = function(arg0, arg1, arg2) {
        arg0.depthRange(arg1, arg2);
    };
    imports.wbg.__wbg_depthRange_7025983a507dd522 = function(arg0, arg1, arg2) {
        arg0.depthRange(arg1, arg2);
    };
    imports.wbg.__wbg_devicePixelRatio_390dee26c70aa30f = function(arg0) {
        const ret = arg0.devicePixelRatio;
        return ret;
    };
    imports.wbg.__wbg_disableVertexAttribArray_4c5c7214724209d0 = function(arg0, arg1) {
        arg0.disableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_disableVertexAttribArray_bcf2272b428ec9fc = function(arg0, arg1) {
        arg0.disableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_disable_3af3e194392b0a83 = function(arg0, arg1) {
        arg0.disable(arg1 >>> 0);
    };
    imports.wbg.__wbg_disable_c05809e00765548d = function(arg0, arg1) {
        arg0.disable(arg1 >>> 0);
    };
    imports.wbg.__wbg_documentElement_39f40310398a4cba = function(arg0) {
        const ret = arg0.documentElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_document_5b745e82ba551ca5 = function(arg0) {
        const ret = arg0.document;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_drawArraysInstancedANGLE_5802f710395d6947 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_drawArraysInstanced_5a3cccf98d769264 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_drawArrays_a8ad03dae79ec56f = function(arg0, arg1, arg2, arg3) {
        arg0.drawArrays(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_drawArrays_c106ebe0234971d4 = function(arg0, arg1, arg2, arg3) {
        arg0.drawArrays(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_drawBuffersWEBGL_f21a161dc8fb366c = function(arg0, arg1) {
        arg0.drawBuffersWEBGL(arg1);
    };
    imports.wbg.__wbg_drawBuffers_dd9a3530aa5b71b2 = function(arg0, arg1) {
        arg0.drawBuffers(arg1);
    };
    imports.wbg.__wbg_drawElementsInstancedANGLE_a63eca97c72be45f = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_drawElementsInstanced_ad84faddf2b48335 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_draw_e8c430e7254c6215 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    };
    imports.wbg.__wbg_enableVertexAttribArray_2898de871f949393 = function(arg0, arg1) {
        arg0.enableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_enableVertexAttribArray_def9952d8426be95 = function(arg0, arg1) {
        arg0.enableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_enable_2d8bb952637ad17a = function(arg0, arg1) {
        arg0.enable(arg1 >>> 0);
    };
    imports.wbg.__wbg_enable_52598759008d46ee = function(arg0, arg1) {
        arg0.enable(arg1 >>> 0);
    };
    imports.wbg.__wbg_endQuery_81a855457c9a8807 = function(arg0, arg1) {
        arg0.endQuery(arg1 >>> 0);
    };
    imports.wbg.__wbg_end_7ad26f2083234d67 = function(arg0) {
        arg0.end();
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free_command_export(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_error_7bc7d576a6aaf855 = function(arg0) {
        console.error(arg0);
    };
    imports.wbg.__wbg_fenceSync_ae9efe266c01d1d4 = function(arg0, arg1, arg2) {
        const ret = arg0.fenceSync(arg1 >>> 0, arg2 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_finish_ac8e8f8408208d93 = function(arg0) {
        const ret = arg0.finish();
        return ret;
    };
    imports.wbg.__wbg_finish_b79779da004ef346 = function(arg0, arg1) {
        const ret = arg0.finish(arg1);
        return ret;
    };
    imports.wbg.__wbg_focus_220a53e22147dc0f = function() { return handleError(function (arg0) {
        arg0.focus();
    }, arguments) };
    imports.wbg.__wbg_framebufferRenderbuffer_c4e0a3741080e47d = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
    };
    imports.wbg.__wbg_framebufferRenderbuffer_d11b93c15d813b67 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
    };
    imports.wbg.__wbg_framebufferTexture2D_1c59ad9667ea1ea1 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_framebufferTexture2D_489e539476d29f49 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_framebufferTextureLayer_adaeec76c62e2293 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
    };
    imports.wbg.__wbg_framebufferTextureMultiviewOVR_81e594036296c9b0 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5, arg6);
    };
    imports.wbg.__wbg_frontFace_9a8e14be7e21500f = function(arg0, arg1) {
        arg0.frontFace(arg1 >>> 0);
    };
    imports.wbg.__wbg_frontFace_b516366b32ef6f00 = function(arg0, arg1) {
        arg0.frontFace(arg1 >>> 0);
    };
    imports.wbg.__wbg_fullscreenElement_e2e939644adf50e1 = function(arg0) {
        const ret = arg0.fullscreenElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_getBoundingClientRect_25e44a78507968b0 = function(arg0) {
        const ret = arg0.getBoundingClientRect();
        return ret;
    };
    imports.wbg.__wbg_getBufferSubData_1867a1050f5a6726 = function(arg0, arg1, arg2, arg3) {
        arg0.getBufferSubData(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_getContext_01f42b234e833f0a = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_1a6877af6b5f04dc = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_2f210d0a58d43d95 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_40a6fc6da6cacc21 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getCurrentTexture_3c8710ca6e0019fc = function() { return handleError(function (arg0) {
        const ret = arg0.getCurrentTexture();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getExtension_49a13df0dc150fab = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getExtension(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getIndexedParameter_46abff0edb598e22 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getMappedRange_86d4a434bceeb7fc = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getMappedRange(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getModifierState_44e56c7d69a223e8 = function(arg0, arg1, arg2) {
        const ret = arg0.getModifierState(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_getParameter_08df3cb47d357cca = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.getParameter(arg1 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getParameter_1dfd667c33169fab = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.getParameter(arg1 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getPreferredCanvasFormat_0988752050c788b0 = function(arg0) {
        const ret = arg0.getPreferredCanvasFormat();
        return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 96) - 1;
    };
    imports.wbg.__wbg_getProgramInfoLog_a0ff8b0971fcaf48 = function(arg0, arg1, arg2) {
        const ret = arg1.getProgramInfoLog(arg2);
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getProgramInfoLog_ea3064b153e4542a = function(arg0, arg1, arg2) {
        const ret = arg1.getProgramInfoLog(arg2);
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getProgramParameter_c777611a448a6ccd = function(arg0, arg1, arg2) {
        const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getProgramParameter_ff1aee3815d6a8f9 = function(arg0, arg1, arg2) {
        const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getQueryParameter_7f1971af9b820343 = function(arg0, arg1, arg2) {
        const ret = arg0.getQueryParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getShaderInfoLog_1affea8c74bd191c = function(arg0, arg1, arg2) {
        const ret = arg1.getShaderInfoLog(arg2);
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getShaderInfoLog_862d8c35c68d02c8 = function(arg0, arg1, arg2) {
        const ret = arg1.getShaderInfoLog(arg2);
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getShaderParameter_1f86483b99db3dcc = function(arg0, arg1, arg2) {
        const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getShaderParameter_b8a41abb0d7d23c3 = function(arg0, arg1, arg2) {
        const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getSupportedExtensions_bc23bc19c9dac45d = function(arg0) {
        const ret = arg0.getSupportedExtensions();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_getSupportedProfiles_d5636f8d10765e75 = function(arg0) {
        const ret = arg0.getSupportedProfiles();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_getSyncParameter_20391c81e5e58c48 = function(arg0, arg1, arg2) {
        const ret = arg0.getSyncParameter(arg1, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getUniformBlockIndex_1453ff945a9eefd5 = function(arg0, arg1, arg2, arg3) {
        const ret = arg0.getUniformBlockIndex(arg1, getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_getUniformLocation_21ac12bfc569cbbf = function(arg0, arg1, arg2, arg3) {
        const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_getUniformLocation_2a4ddf8dd8285373 = function(arg0, arg1, arg2, arg3) {
        const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_get_6b7bd52aca3f9671 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_get_84957f57555c0874 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_get_c53d381635aa3929 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_gpu_051bdce6489ddf6a = function(arg0) {
        const ret = arg0.gpu;
        return ret;
    };
    imports.wbg.__wbg_hasFocus_7d4687baa7939850 = function() { return handleError(function (arg0) {
        const ret = arg0.hasFocus();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_height_5d22b94a936fae9f = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_hidden_63c9db3ea5c1e10a = function(arg0) {
        const ret = arg0.hidden;
        return ret;
    };
    imports.wbg.__wbg_includes_cd7103de1f6ce823 = function(arg0, arg1, arg2) {
        const ret = arg0.includes(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_innerHeight_ccac894168ddb23c = function() { return handleError(function (arg0) {
        const ret = arg0.innerHeight;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_innerWidth_5a44c76afe8ca065 = function() { return handleError(function (arg0) {
        const ret = arg0.innerWidth;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuAdapter_aff4b0f95a6c1c3e = function(arg0) {
        let result;
        try {
            result = arg0 instanceof GPUAdapter;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_GpuCanvasContext_dc8dc7061b962990 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof GPUCanvasContext;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlCanvasElement_c4251b1b6a15edcc = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLCanvasElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlInputElement_46b31917ce88698f = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLInputElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_ResizeObserverEntry_598d7b7088f31bb9 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof ResizeObserverEntry;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_WebGl2RenderingContext_121e4c8c95b128ef = function(arg0) {
        let result;
        try {
            result = arg0 instanceof WebGL2RenderingContext;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_b5cf7783caa68180 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_invalidateFramebuffer_e2d4d1747d73b885 = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.invalidateFramebuffer(arg1 >>> 0, arg2);
    }, arguments) };
    imports.wbg.__wbg_isComposing_fcde9aa6cddb1f42 = function(arg0) {
        const ret = arg0.isComposing;
        return ret;
    };
    imports.wbg.__wbg_is_928aa29d71e75457 = function(arg0, arg1) {
        const ret = Object.is(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_key_505d33c50799526a = function(arg0, arg1) {
        const ret = arg1.key;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_label_c3a930571192f18e = function(arg0, arg1) {
        const ret = arg1.label;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_language_763ea76470ed849b = function(arg0, arg1) {
        const ret = arg1.language;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_left_d52bfa3824286825 = function(arg0) {
        const ret = arg0.left;
        return ret;
    };
    imports.wbg.__wbg_length_22ac23eaec9d8053 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_7226f74c66ca4a9d = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_d45040a40c570362 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_linkProgram_2f770464e69099dc = function(arg0, arg1) {
        arg0.linkProgram(arg1);
    };
    imports.wbg.__wbg_linkProgram_93f76a2f5030041e = function(arg0, arg1) {
        arg0.linkProgram(arg1);
    };
    imports.wbg.__wbg_location_962e75c1c1b3ebed = function(arg0) {
        const ret = arg0.location;
        return ret;
    };
    imports.wbg.__wbg_log_1d990106d99dacb7 = function(arg0) {
        console.log(arg0);
    };
    imports.wbg.__wbg_matchMedia_29904c79dbaba90b = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.matchMedia(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_matches_9cef9b7c722bd7c8 = function(arg0) {
        const ret = arg0.matches;
        return ret;
    };
    imports.wbg.__wbg_metaKey_0572b1cbcb5b272b = function(arg0) {
        const ret = arg0.metaKey;
        return ret;
    };
    imports.wbg.__wbg_metaKey_448c751accad2eba = function(arg0) {
        const ret = arg0.metaKey;
        return ret;
    };
    imports.wbg.__wbg_navigator_11b7299bb7886507 = function(arg0) {
        const ret = arg0.navigator;
        return ret;
    };
    imports.wbg.__wbg_navigator_b49edef831236138 = function(arg0) {
        const ret = arg0.navigator;
        return ret;
    };
    imports.wbg.__wbg_new_1ba21ce319a06297 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_25f239778d6112b9 = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_new_a25bd305a87faf63 = function() { return handleError(function (arg0) {
        const ret = new ResizeObserver(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_from_slice_f9c22b9153b26992 = function(arg0, arg1) {
        const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_no_args_cb138f77cf6151ee = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_with_byte_offset_and_length_d85c3da1fd8df149 = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_now_2c95c9de01293173 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_now_69d776cd24f5215b = function() {
        const ret = Date.now();
        return ret;
    };
    imports.wbg.__wbg_observe_eefa2465578e5d51 = function(arg0, arg1) {
        arg0.observe(arg1);
    };
    imports.wbg.__wbg_of_6505a0eb509da02e = function(arg0) {
        const ret = Array.of(arg0);
        return ret;
    };
    imports.wbg.__wbg_offsetX_cef943cf53ab2b5a = function(arg0) {
        const ret = arg0.offsetX;
        return ret;
    };
    imports.wbg.__wbg_offsetY_9a093457f71ef493 = function(arg0) {
        const ret = arg0.offsetY;
        return ret;
    };
    imports.wbg.__wbg_open_c565053c17d497bd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.open(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_performance_7a3ffd0b17f663ad = function(arg0) {
        const ret = arg0.performance;
        return ret;
    };
    imports.wbg.__wbg_pixelStorei_1956db9ae4b22c29 = function(arg0, arg1, arg2) {
        arg0.pixelStorei(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_pixelStorei_5449c87f83f25694 = function(arg0, arg1, arg2) {
        arg0.pixelStorei(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_polygonOffset_7308f17e4b9c9e6f = function(arg0, arg1, arg2) {
        arg0.polygonOffset(arg1, arg2);
    };
    imports.wbg.__wbg_polygonOffset_d405a847eb9279a1 = function(arg0, arg1, arg2) {
        arg0.polygonOffset(arg1, arg2);
    };
    imports.wbg.__wbg_preventDefault_e97663aeeb9709d3 = function(arg0) {
        arg0.preventDefault();
    };
    imports.wbg.__wbg_prototypesetcall_dfe9b766cdc1f1fd = function(arg0, arg1, arg2) {
        Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    };
    imports.wbg.__wbg_push_7d9be8f38fc13975 = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_queryCounterEXT_ecccc67a3c00d9b2 = function(arg0, arg1, arg2) {
        arg0.queryCounterEXT(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_querySelectorAll_aa1048eae18f6f1a = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_querySelector_15a92ce6bed6157d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_9b549dfce8865860 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_fca69f5bfad613a5 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queue_1f589e8194b004a6 = function(arg0) {
        const ret = arg0.queue;
        return ret;
    };
    imports.wbg.__wbg_readBuffer_bbd823c99c8cb8c2 = function(arg0, arg1) {
        arg0.readBuffer(arg1 >>> 0);
    };
    imports.wbg.__wbg_readPixels_031b1d4c916fc4f9 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
    }, arguments) };
    imports.wbg.__wbg_readPixels_3288aabda6ab89ff = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
    }, arguments) };
    imports.wbg.__wbg_readPixels_bc06772e95599959 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
    }, arguments) };
    imports.wbg.__wbg_reload_27ff3c39a5227750 = function() { return handleError(function (arg0) {
        arg0.reload();
    }, arguments) };
    imports.wbg.__wbg_renderbufferStorageMultisample_c944aa96428a6ff6 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_renderbufferStorage_4ea9706d7f996e6d = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
    };
    imports.wbg.__wbg_renderbufferStorage_95fae6488cee51e3 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
    };
    imports.wbg.__wbg_repeat_3733d1d584bf0e38 = function(arg0) {
        const ret = arg0.repeat;
        return ret;
    };
    imports.wbg.__wbg_requestAdapter_51be7e8ee7d08b87 = function(arg0, arg1) {
        const ret = arg0.requestAdapter(arg1);
        return ret;
    };
    imports.wbg.__wbg_requestAnimationFrame_994dc4ebde22b8d9 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.requestAnimationFrame(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requestDevice_338f0085866d40a2 = function(arg0, arg1) {
        const ret = arg0.requestDevice(arg1);
        return ret;
    };
    imports.wbg.__wbg_requestFullscreen_0d9f7148d4658c31 = function() { return handleError(function (arg0) {
        arg0.requestFullscreen();
    }, arguments) };
    imports.wbg.__wbg_resolve_fd5bfbaa4ce36e1e = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_samplerParameterf_dc4f26238b36d07a = function(arg0, arg1, arg2, arg3) {
        arg0.samplerParameterf(arg1, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_samplerParameteri_66d42118f12ed70c = function(arg0, arg1, arg2, arg3) {
        arg0.samplerParameteri(arg1, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_scissor_04e903bd18e45083 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.scissor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_scissor_988df87f9cf85e7e = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.scissor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_setAttribute_34747dd193f45828 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_306b5f43159153da = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_d3cd0c65d5718e66 = function(arg0, arg1, arg2) {
        arg0.setBindGroup(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_setPipeline_f44bbc63b7455235 = function(arg0, arg1) {
        arg0.setPipeline(arg1);
    };
    imports.wbg.__wbg_setProperty_f27b2c05323daf8a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setTimeout_06477c23d31efef1 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.setTimeout(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_5e5ec203042c0564 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_setVertexBuffer_950908f301fc83b4 = function(arg0, arg1, arg2, arg3) {
        arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_set_781438a03c0c3c81 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_a_6ca4b80abcaa9bb0 = function(arg0, arg1) {
        arg0.a = arg1;
    };
    imports.wbg.__wbg_set_access_c17e0a436ed1d78e = function(arg0, arg1) {
        arg0.access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
    };
    imports.wbg.__wbg_set_address_mode_u_9a2648489304b6c3 = function(arg0, arg1) {
        arg0.addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
    };
    imports.wbg.__wbg_set_address_mode_v_911f607ff1319cf6 = function(arg0, arg1) {
        arg0.addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
    };
    imports.wbg.__wbg_set_address_mode_w_b7c68665b89d5500 = function(arg0, arg1) {
        arg0.addressModeW = __wbindgen_enum_GpuAddressMode[arg1];
    };
    imports.wbg.__wbg_set_alpha_eb6e37beb08f6a6a = function(arg0, arg1) {
        arg0.alpha = arg1;
    };
    imports.wbg.__wbg_set_alpha_mode_2a9be051489d8bbd = function(arg0, arg1) {
        arg0.alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
    };
    imports.wbg.__wbg_set_alpha_to_coverage_enabled_1f594c6ef9ae4caa = function(arg0, arg1) {
        arg0.alphaToCoverageEnabled = arg1 !== 0;
    };
    imports.wbg.__wbg_set_array_layer_count_93d58eca9387b84c = function(arg0, arg1) {
        arg0.arrayLayerCount = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_array_stride_5ace211a6c31af55 = function(arg0, arg1) {
        arg0.arrayStride = arg1;
    };
    imports.wbg.__wbg_set_aspect_e3aa9cad44e6338f = function(arg0, arg1) {
        arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
    };
    imports.wbg.__wbg_set_attributes_8cfe8a349778ff6d = function(arg0, arg1) {
        arg0.attributes = arg1;
    };
    imports.wbg.__wbg_set_b_52915cc78721cadb = function(arg0, arg1) {
        arg0.b = arg1;
    };
    imports.wbg.__wbg_set_base_array_layer_798dcd012d28aafd = function(arg0, arg1) {
        arg0.baseArrayLayer = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_base_mip_level_ff05f0742029fbd7 = function(arg0, arg1) {
        arg0.baseMipLevel = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_bc3a432bdcd60886 = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_beginning_of_pass_write_index_ad07a73147217513 = function(arg0, arg1) {
        arg0.beginningOfPassWriteIndex = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_bind_group_layouts_9eff5e187a1db39e = function(arg0, arg1) {
        arg0.bindGroupLayouts = arg1;
    };
    imports.wbg.__wbg_set_binding_3ada8a83c514d419 = function(arg0, arg1) {
        arg0.binding = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_binding_9a389db987313ca9 = function(arg0, arg1) {
        arg0.binding = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_blend_15fcdb6fca391aa3 = function(arg0, arg1) {
        arg0.blend = arg1;
    };
    imports.wbg.__wbg_set_buffer_581ee8422928bd0d = function(arg0, arg1) {
        arg0.buffer = arg1;
    };
    imports.wbg.__wbg_set_buffer_ac25c198252221bd = function(arg0, arg1) {
        arg0.buffer = arg1;
    };
    imports.wbg.__wbg_set_buffers_4515e14c72e1bc45 = function(arg0, arg1) {
        arg0.buffers = arg1;
    };
    imports.wbg.__wbg_set_bytes_per_row_4c52e94a64f7b18a = function(arg0, arg1) {
        arg0.bytesPerRow = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_clear_value_9fd25161e3ff7358 = function(arg0, arg1) {
        arg0.clearValue = arg1;
    };
    imports.wbg.__wbg_set_code_1d146372551ab97f = function(arg0, arg1, arg2) {
        arg0.code = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_color_63a788c8828014d8 = function(arg0, arg1) {
        arg0.color = arg1;
    };
    imports.wbg.__wbg_set_color_attachments_b56ec268556eb0af = function(arg0, arg1) {
        arg0.colorAttachments = arg1;
    };
    imports.wbg.__wbg_set_compare_986db63daac4c337 = function(arg0, arg1) {
        arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
    };
    imports.wbg.__wbg_set_compare_b6bd133fd1c7206a = function(arg0, arg1) {
        arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
    };
    imports.wbg.__wbg_set_count_6b3574238f446a02 = function(arg0, arg1) {
        arg0.count = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_cull_mode_f1cc439f208cf7d2 = function(arg0, arg1) {
        arg0.cullMode = __wbindgen_enum_GpuCullMode[arg1];
    };
    imports.wbg.__wbg_set_depth_bias_0c225de07a2372b1 = function(arg0, arg1) {
        arg0.depthBias = arg1;
    };
    imports.wbg.__wbg_set_depth_bias_clamp_bd34181bc74b8a65 = function(arg0, arg1) {
        arg0.depthBiasClamp = arg1;
    };
    imports.wbg.__wbg_set_depth_bias_slope_scale_d43ddce65f19c9be = function(arg0, arg1) {
        arg0.depthBiasSlopeScale = arg1;
    };
    imports.wbg.__wbg_set_depth_clear_value_eb76fedd34b20053 = function(arg0, arg1) {
        arg0.depthClearValue = arg1;
    };
    imports.wbg.__wbg_set_depth_compare_491947ed2f6065b9 = function(arg0, arg1) {
        arg0.depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
    };
    imports.wbg.__wbg_set_depth_fail_op_4983b01413b9f743 = function(arg0, arg1) {
        arg0.depthFailOp = __wbindgen_enum_GpuStencilOperation[arg1];
    };
    imports.wbg.__wbg_set_depth_load_op_c7deb718c4129a2c = function(arg0, arg1) {
        arg0.depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
    };
    imports.wbg.__wbg_set_depth_or_array_layers_5686e74657700bc2 = function(arg0, arg1) {
        arg0.depthOrArrayLayers = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_depth_read_only_18602250b14fa638 = function(arg0, arg1) {
        arg0.depthReadOnly = arg1 !== 0;
    };
    imports.wbg.__wbg_set_depth_stencil_attachment_90d13c414095197d = function(arg0, arg1) {
        arg0.depthStencilAttachment = arg1;
    };
    imports.wbg.__wbg_set_depth_stencil_e6069a8b511d1004 = function(arg0, arg1) {
        arg0.depthStencil = arg1;
    };
    imports.wbg.__wbg_set_depth_store_op_55f84f2f9039c453 = function(arg0, arg1) {
        arg0.depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
    };
    imports.wbg.__wbg_set_depth_write_enabled_e419ffe553654371 = function(arg0, arg1) {
        arg0.depthWriteEnabled = arg1 !== 0;
    };
    imports.wbg.__wbg_set_device_91facdf766d51abf = function(arg0, arg1) {
        arg0.device = arg1;
    };
    imports.wbg.__wbg_set_dimension_47ad758bb7805028 = function(arg0, arg1) {
        arg0.dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    };
    imports.wbg.__wbg_set_dimension_500c3bec57e8ac12 = function(arg0, arg1) {
        arg0.dimension = __wbindgen_enum_GpuTextureDimension[arg1];
    };
    imports.wbg.__wbg_set_dst_factor_abdf4d85b8f742b5 = function(arg0, arg1) {
        arg0.dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
    };
    imports.wbg.__wbg_set_end_of_pass_write_index_82a42f6ec7d55754 = function(arg0, arg1) {
        arg0.endOfPassWriteIndex = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_entries_136baaaafb25087f = function(arg0, arg1) {
        arg0.entries = arg1;
    };
    imports.wbg.__wbg_set_entries_7c41d594195ebe78 = function(arg0, arg1) {
        arg0.entries = arg1;
    };
    imports.wbg.__wbg_set_entry_point_913e091cc9a07667 = function(arg0, arg1, arg2) {
        arg0.entryPoint = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_entry_point_96944272d50efb55 = function(arg0, arg1, arg2) {
        arg0.entryPoint = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_fail_op_fd94b46d0cd7c4f2 = function(arg0, arg1) {
        arg0.failOp = __wbindgen_enum_GpuStencilOperation[arg1];
    };
    imports.wbg.__wbg_set_format_29126ee763612515 = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_format_450c4be578985cb4 = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuVertexFormat[arg1];
    };
    imports.wbg.__wbg_set_format_582f639b8a79115c = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_format_6ac892268eeef402 = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_format_a622a57e42ae23e4 = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_format_bdfc7be2aa989382 = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_format_c3ba1e26468014ae = function(arg0, arg1) {
        arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    };
    imports.wbg.__wbg_set_fragment_84f03cfa83c432b2 = function(arg0, arg1) {
        arg0.fragment = arg1;
    };
    imports.wbg.__wbg_set_front_face_1c87b2e21f85a97f = function(arg0, arg1) {
        arg0.frontFace = __wbindgen_enum_GpuFrontFace[arg1];
    };
    imports.wbg.__wbg_set_g_b94c63958617b86c = function(arg0, arg1) {
        arg0.g = arg1;
    };
    imports.wbg.__wbg_set_has_dynamic_offset_9dc29179158975e4 = function(arg0, arg1) {
        arg0.hasDynamicOffset = arg1 !== 0;
    };
    imports.wbg.__wbg_set_height_080fa3e226a83750 = function(arg0, arg1) {
        arg0.height = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_height_6f8f8ef4cb40e496 = function(arg0, arg1) {
        arg0.height = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_height_afe09c24165867f7 = function(arg0, arg1) {
        arg0.height = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_id_702da6e1bcec3b45 = function(arg0, arg1, arg2) {
        arg0.id = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_034d85243342ac5c = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_1e2e0069cbf2bd78 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_21544401e31cd317 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_2312a64e22934a2b = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_2ed86217d97ea3d5 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_3f988ca8291e319f = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_73d706a16d13a23c = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_81dd67dee9cd4287 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_8f9ebe053f8da7a0 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_bfbd23fc748f8f94 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_d400966bd7759b26 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_e1499888d936ca3f = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_label_ecb2c1eab1d46433 = function(arg0, arg1, arg2) {
        arg0.label = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_layout_0770a97fe3411616 = function(arg0, arg1) {
        arg0.layout = arg1;
    };
    imports.wbg.__wbg_set_layout_0e88cce0b3d76c31 = function(arg0, arg1) {
        arg0.layout = arg1;
    };
    imports.wbg.__wbg_set_load_op_6725bf0c5b509ae7 = function(arg0, arg1) {
        arg0.loadOp = __wbindgen_enum_GpuLoadOp[arg1];
    };
    imports.wbg.__wbg_set_lod_max_clamp_3a51dd81fde72c8d = function(arg0, arg1) {
        arg0.lodMaxClamp = arg1;
    };
    imports.wbg.__wbg_set_lod_min_clamp_f48943c1f01e12f9 = function(arg0, arg1) {
        arg0.lodMinClamp = arg1;
    };
    imports.wbg.__wbg_set_mag_filter_5794fd33d3902192 = function(arg0, arg1) {
        arg0.magFilter = __wbindgen_enum_GpuFilterMode[arg1];
    };
    imports.wbg.__wbg_set_mapped_at_creation_e0c884a30f64323b = function(arg0, arg1) {
        arg0.mappedAtCreation = arg1 !== 0;
    };
    imports.wbg.__wbg_set_mask_9094d3e3f6f3a7dc = function(arg0, arg1) {
        arg0.mask = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_max_anisotropy_1377b74addad8758 = function(arg0, arg1) {
        arg0.maxAnisotropy = arg1;
    };
    imports.wbg.__wbg_set_min_binding_size_4a9f4d0d9ee579af = function(arg0, arg1) {
        arg0.minBindingSize = arg1;
    };
    imports.wbg.__wbg_set_min_filter_32dc39202a18cd7b = function(arg0, arg1) {
        arg0.minFilter = __wbindgen_enum_GpuFilterMode[arg1];
    };
    imports.wbg.__wbg_set_mip_level_992f82e991b163b8 = function(arg0, arg1) {
        arg0.mipLevel = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_mip_level_count_1d13855f7726190c = function(arg0, arg1) {
        arg0.mipLevelCount = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_mip_level_count_a5a0102e4248e5bb = function(arg0, arg1) {
        arg0.mipLevelCount = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_mipmap_filter_00493c30d94b571e = function(arg0, arg1) {
        arg0.mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
    };
    imports.wbg.__wbg_set_module_882651860e912779 = function(arg0, arg1) {
        arg0.module = arg1;
    };
    imports.wbg.__wbg_set_module_b46c4a937ee89c3b = function(arg0, arg1) {
        arg0.module = arg1;
    };
    imports.wbg.__wbg_set_multisample_0a38af2e310bacc6 = function(arg0, arg1) {
        arg0.multisample = arg1;
    };
    imports.wbg.__wbg_set_multisampled_f2de771b3ad62ff3 = function(arg0, arg1) {
        arg0.multisampled = arg1 !== 0;
    };
    imports.wbg.__wbg_set_offset_31c0a660f535c545 = function(arg0, arg1) {
        arg0.offset = arg1;
    };
    imports.wbg.__wbg_set_offset_3eb0797dcc9c9464 = function(arg0, arg1) {
        arg0.offset = arg1;
    };
    imports.wbg.__wbg_set_offset_a675629849c5f3b4 = function(arg0, arg1) {
        arg0.offset = arg1;
    };
    imports.wbg.__wbg_set_operation_879618283d591339 = function(arg0, arg1) {
        arg0.operation = __wbindgen_enum_GpuBlendOperation[arg1];
    };
    imports.wbg.__wbg_set_origin_11de57058b4d23fb = function(arg0, arg1) {
        arg0.origin = arg1;
    };
    imports.wbg.__wbg_set_pass_op_238c7cbc20505ae9 = function(arg0, arg1) {
        arg0.passOp = __wbindgen_enum_GpuStencilOperation[arg1];
    };
    imports.wbg.__wbg_set_power_preference_f4cead100f48bab0 = function(arg0, arg1) {
        arg0.powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
    };
    imports.wbg.__wbg_set_primitive_01150af3e98fb372 = function(arg0, arg1) {
        arg0.primitive = arg1;
    };
    imports.wbg.__wbg_set_query_set_8441106911a3af36 = function(arg0, arg1) {
        arg0.querySet = arg1;
    };
    imports.wbg.__wbg_set_r_08c1678b22216ee0 = function(arg0, arg1) {
        arg0.r = arg1;
    };
    imports.wbg.__wbg_set_required_features_e9ee2e22feba0db3 = function(arg0, arg1) {
        arg0.requiredFeatures = arg1;
    };
    imports.wbg.__wbg_set_resolve_target_d00e2ef5a7388503 = function(arg0, arg1) {
        arg0.resolveTarget = arg1;
    };
    imports.wbg.__wbg_set_resource_5a4cc69a127b394e = function(arg0, arg1) {
        arg0.resource = arg1;
    };
    imports.wbg.__wbg_set_rows_per_image_f456122723767189 = function(arg0, arg1) {
        arg0.rowsPerImage = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_sample_count_c44a2a6eebe72dcc = function(arg0, arg1) {
        arg0.sampleCount = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_sample_type_89fd8e71274ee6c2 = function(arg0, arg1) {
        arg0.sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
    };
    imports.wbg.__wbg_set_sampler_ab33334fb83c5a17 = function(arg0, arg1) {
        arg0.sampler = arg1;
    };
    imports.wbg.__wbg_set_shader_location_b905e964144cc9ad = function(arg0, arg1) {
        arg0.shaderLocation = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_size_a877ed6f434871bd = function(arg0, arg1) {
        arg0.size = arg1;
    };
    imports.wbg.__wbg_set_size_b2cab7e432ec25dc = function(arg0, arg1) {
        arg0.size = arg1;
    };
    imports.wbg.__wbg_set_size_c167af29ed0f618c = function(arg0, arg1) {
        arg0.size = arg1;
    };
    imports.wbg.__wbg_set_src_factor_3bf35cc93f12e8c2 = function(arg0, arg1) {
        arg0.srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
    };
    imports.wbg.__wbg_set_stencil_back_6d0e3812c09eb489 = function(arg0, arg1) {
        arg0.stencilBack = arg1;
    };
    imports.wbg.__wbg_set_stencil_clear_value_53b51b80af22b8a4 = function(arg0, arg1) {
        arg0.stencilClearValue = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_stencil_front_223b59e436e04d2d = function(arg0, arg1) {
        arg0.stencilFront = arg1;
    };
    imports.wbg.__wbg_set_stencil_load_op_d88ff17c1f14f3b3 = function(arg0, arg1) {
        arg0.stencilLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
    };
    imports.wbg.__wbg_set_stencil_read_mask_f7b2d22f2682c8f6 = function(arg0, arg1) {
        arg0.stencilReadMask = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_stencil_read_only_6fba8956bae14007 = function(arg0, arg1) {
        arg0.stencilReadOnly = arg1 !== 0;
    };
    imports.wbg.__wbg_set_stencil_store_op_9637a0cb039fc7bb = function(arg0, arg1) {
        arg0.stencilStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
    };
    imports.wbg.__wbg_set_stencil_write_mask_fc2b202439c71444 = function(arg0, arg1) {
        arg0.stencilWriteMask = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_step_mode_953dbc499c2ea5db = function(arg0, arg1) {
        arg0.stepMode = __wbindgen_enum_GpuVertexStepMode[arg1];
    };
    imports.wbg.__wbg_set_storage_texture_0634dd6c87ac1132 = function(arg0, arg1) {
        arg0.storageTexture = arg1;
    };
    imports.wbg.__wbg_set_store_op_d6e36afb7a3bc15a = function(arg0, arg1) {
        arg0.storeOp = __wbindgen_enum_GpuStoreOp[arg1];
    };
    imports.wbg.__wbg_set_strip_index_format_6813dd6e867de4f2 = function(arg0, arg1) {
        arg0.stripIndexFormat = __wbindgen_enum_GpuIndexFormat[arg1];
    };
    imports.wbg.__wbg_set_targets_0ab03a33d2c15ccd = function(arg0, arg1) {
        arg0.targets = arg1;
    };
    imports.wbg.__wbg_set_texture_72c4d60403590233 = function(arg0, arg1) {
        arg0.texture = arg1;
    };
    imports.wbg.__wbg_set_texture_9dc3759e93cfbb84 = function(arg0, arg1) {
        arg0.texture = arg1;
    };
    imports.wbg.__wbg_set_timestamp_writes_736aa6c2c69ccaea = function(arg0, arg1) {
        arg0.timestampWrites = arg1;
    };
    imports.wbg.__wbg_set_title_68ffc586125a93b4 = function(arg0, arg1, arg2) {
        arg0.title = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_topology_84962f44b37e8986 = function(arg0, arg1) {
        arg0.topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
    };
    imports.wbg.__wbg_set_type_466673d0a1ab874b = function(arg0, arg1, arg2) {
        arg0.type = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_type_4ff365ea9ad896aa = function(arg0, arg1) {
        arg0.type = __wbindgen_enum_GpuBufferBindingType[arg1];
    };
    imports.wbg.__wbg_set_type_b4b2fc6fbad39aeb = function(arg0, arg1) {
        arg0.type = __wbindgen_enum_GpuSamplerBindingType[arg1];
    };
    imports.wbg.__wbg_set_usage_3bf7bce356282919 = function(arg0, arg1) {
        arg0.usage = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_usage_48c9e7b82b575c9a = function(arg0, arg1) {
        arg0.usage = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_usage_a102e6844c6a65de = function(arg0, arg1) {
        arg0.usage = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_usage_ea5e5efc19daea09 = function(arg0, arg1) {
        arg0.usage = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_value_8f487a4f7d71c024 = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_vertex_96327c405a801524 = function(arg0, arg1) {
        arg0.vertex = arg1;
    };
    imports.wbg.__wbg_set_view_2d2806aa6c5822ca = function(arg0, arg1) {
        arg0.view = arg1;
    };
    imports.wbg.__wbg_set_view_b7216eb00b7f584a = function(arg0, arg1) {
        arg0.view = arg1;
    };
    imports.wbg.__wbg_set_view_dimension_c6aedf84f79e2593 = function(arg0, arg1) {
        arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    };
    imports.wbg.__wbg_set_view_dimension_ccb64a21a1495106 = function(arg0, arg1) {
        arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    };
    imports.wbg.__wbg_set_view_formats_65a3ce6335913be2 = function(arg0, arg1) {
        arg0.viewFormats = arg1;
    };
    imports.wbg.__wbg_set_view_formats_d7be9eae49a0933b = function(arg0, arg1) {
        arg0.viewFormats = arg1;
    };
    imports.wbg.__wbg_set_visibility_3445d21752d17ded = function(arg0, arg1) {
        arg0.visibility = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_width_0a22c810f06a5152 = function(arg0, arg1) {
        arg0.width = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_width_7ff7a22c6e9f423e = function(arg0, arg1) {
        arg0.width = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_width_ff3dae6ae4838a9e = function(arg0, arg1) {
        arg0.width = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_write_mask_b94f0c67654d5b00 = function(arg0, arg1) {
        arg0.writeMask = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_x_cb03e4f7e9c6b588 = function(arg0, arg1) {
        arg0.x = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_y_ca78b7606a8f2c0c = function(arg0, arg1) {
        arg0.y = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_z_5389d800d9ef03b4 = function(arg0, arg1) {
        arg0.z = arg1 >>> 0;
    };
    imports.wbg.__wbg_shaderSource_8a7a30baeaf655d5 = function(arg0, arg1, arg2, arg3) {
        arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_shaderSource_aea71cfa376fc985 = function(arg0, arg1, arg2, arg3) {
        arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_shiftKey_a6df227a917d203b = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_shiftKey_d2640abcfa98acec = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_769e6b65d6557335 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_08f5a74c69739274 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_a8924b26aa92d024 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_stencilFuncSeparate_8837ff1279f2bcd8 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
    };
    imports.wbg.__wbg_stencilFuncSeparate_b6b919cb79b36c7f = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
    };
    imports.wbg.__wbg_stencilMaskSeparate_8780b512ad994312 = function(arg0, arg1, arg2) {
        arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_stencilMaskSeparate_fdaf7687ee443945 = function(arg0, arg1, arg2) {
        arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_stencilMask_729d1b04c4560c92 = function(arg0, arg1) {
        arg0.stencilMask(arg1 >>> 0);
    };
    imports.wbg.__wbg_stencilMask_8763a80561b98dde = function(arg0, arg1) {
        arg0.stencilMask(arg1 >>> 0);
    };
    imports.wbg.__wbg_stencilOpSeparate_126147c7d73a0e8e = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    };
    imports.wbg.__wbg_stencilOpSeparate_d1770154b137259f = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    };
    imports.wbg.__wbg_style_521a717da50e53c6 = function(arg0) {
        const ret = arg0.style;
        return ret;
    };
    imports.wbg.__wbg_submit_522f9e0b9d7e22fd = function(arg0, arg1) {
        arg0.submit(arg1);
    };
    imports.wbg.__wbg_texImage2D_9626e500f8562784 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texImage2D_d2480404caf2a35b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texImage2D_d37b35cd7e971b0d = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texImage3D_0c45150b4a96b45e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
    }, arguments) };
    imports.wbg.__wbg_texImage3D_0c9cf74f3c3c59fe = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
    }, arguments) };
    imports.wbg.__wbg_texParameteri_035e104616b395e0 = function(arg0, arg1, arg2, arg3) {
        arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_texParameteri_3a52bfd2ef280632 = function(arg0, arg1, arg2, arg3) {
        arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_texStorage2D_21e779f76539549d = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_texStorage3D_0b08c3a68b3d128e = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
    };
    imports.wbg.__wbg_texSubImage2D_1f2ed8e2272ea41a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_38b182399f10128e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_65b65c3b76d83400 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_6b92ceb1553771fc = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_7b89a7441b2a9257 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_b3a850c16797a6b2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_dc6a2bd41673ac84 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_dc95b375d770251c = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_60e409379482084f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_78f029ad7e55ca39 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_9f46bb4a0a79d9e3 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_a86271ca5befc16d = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_af2ddc81a17c35ce = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_da65e56061783a1b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_e878e89d319561b4 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_then_429f7caf1026411d = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_then_4f95312d68691235 = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_top_7d5b82a2c5d7f13f = function(arg0) {
        const ret = arg0.top;
        return ret;
    };
    imports.wbg.__wbg_uniform1f_058417475b9966c8 = function(arg0, arg1, arg2) {
        arg0.uniform1f(arg1, arg2);
    };
    imports.wbg.__wbg_uniform1f_b47da9590d2c2cf1 = function(arg0, arg1, arg2) {
        arg0.uniform1f(arg1, arg2);
    };
    imports.wbg.__wbg_uniform1i_85131b7388bc8e3f = function(arg0, arg1, arg2) {
        arg0.uniform1i(arg1, arg2);
    };
    imports.wbg.__wbg_uniform1i_e48736e68cd30ed1 = function(arg0, arg1, arg2) {
        arg0.uniform1i(arg1, arg2);
    };
    imports.wbg.__wbg_uniform1ui_03b9da58a76f91cf = function(arg0, arg1, arg2) {
        arg0.uniform1ui(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_uniform2fv_708680e0e9752754 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform2fv_908e28848891e2bf = function(arg0, arg1, arg2, arg3) {
        arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform2iv_3f71696540a8b2ea = function(arg0, arg1, arg2, arg3) {
        arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform2iv_a0cc429953135311 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform2uiv_8b142338906d7ff5 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform2uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform3fv_aa655890f3512e6b = function(arg0, arg1, arg2, arg3) {
        arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform3fv_e58ff84eca16cad5 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform3iv_624ea88531cdde63 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform3iv_afc54662b2809357 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform3uiv_ff68240586289823 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform3uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform4f_1e4aad4d202f9f6c = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
    };
    imports.wbg.__wbg_uniform4f_f0ae29c4c1eb79e0 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
    };
    imports.wbg.__wbg_uniform4fv_2521ae2ffe6e215c = function(arg0, arg1, arg2, arg3) {
        arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform4fv_9913dec8e48633d9 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform4iv_6d0331d24af48aea = function(arg0, arg1, arg2, arg3) {
        arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform4iv_9e38dad2e14636c0 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniform4uiv_766efbfa63685f92 = function(arg0, arg1, arg2, arg3) {
        arg0.uniform4uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_uniformBlockBinding_83eb9ed3f1189da9 = function(arg0, arg1, arg2, arg3) {
        arg0.uniformBlockBinding(arg1, arg2 >>> 0, arg3 >>> 0);
    };
    imports.wbg.__wbg_uniformMatrix2fv_13787967d812a489 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix2fv_90702a9a8694e69b = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix2x3fv_f0dad33c79231b14 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix2x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix2x4fv_c11cac98bdf0e214 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix2x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix3fv_3b2ed3a816d45543 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix3fv_eb9d7317ce9cb6b5 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix3x2fv_11d50f0b78d73578 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix3x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix3x4fv_a78caffb62d235c9 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix3x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix4fv_54fea58f845bbc0e = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix4fv_62e9aaf2b4268690 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix4x2fv_692f10b2150c1ef9 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix4x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_uniformMatrix4x3fv_45f7b122755e52e6 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.uniformMatrix4x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_unmap_a7fc4fb3238304a4 = function(arg0) {
        arg0.unmap();
    };
    imports.wbg.__wbg_useProgram_142dd02d095f80f1 = function(arg0, arg1) {
        arg0.useProgram(arg1);
    };
    imports.wbg.__wbg_useProgram_4632a62f19deea67 = function(arg0, arg1) {
        arg0.useProgram(arg1);
    };
    imports.wbg.__wbg_vertexAttribDivisorANGLE_0797a329758e2a28 = function(arg0, arg1, arg2) {
        arg0.vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_vertexAttribDivisor_4f37e0f7c1197d16 = function(arg0, arg1, arg2) {
        arg0.vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_vertexAttribIPointer_87d7fcce484093c9 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    };
    imports.wbg.__wbg_vertexAttribPointer_5c516f4c675103bf = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    };
    imports.wbg.__wbg_vertexAttribPointer_880223685613a791 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    };
    imports.wbg.__wbg_viewport_1b0f7b63c424b52f = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.viewport(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_viewport_ceaa5c1a061b76df = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.viewport(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_warn_6e567d0d926ff881 = function(arg0) {
        console.warn(arg0);
    };
    imports.wbg.__wbg_width_30d712cfe70e4fae = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_writeText_c9776abb6826901c = function(arg0, arg1, arg2) {
        const ret = arg0.writeText(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_writeTexture_2f9937d7cf0d5da0 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.writeTexture(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbindgen_cast_003358f398b604df = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("CompositionEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_07ff80afce306fe9 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [], shim_idx: 865, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke______);
        return ret;
    };
    imports.wbg.__wbindgen_cast_0a2f83ce3aae7f40 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("WheelEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_11d59837d5a2b494 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("Array<any>")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_460c89d904061ffb = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_503af9d9b47406db = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("MediaQueryListEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_6a9b6c3257c3208c = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 3367, function: Function { arguments: [Externref], shim_idx: 3368, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__wasm_bindgen_795bb83e9b24f476___JsValue____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___wasm_bindgen_795bb83e9b24f476___JsValue_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_7170d0b30c6a50bd = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_7c316abdc43840a3 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(U32)) -> NamedExternref("Uint32Array")`.
        const ret = getArrayU32FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_83e8a979af696361 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_86d5be5481e6f325 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("InputEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_9575fb55a66c262b = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(I32)) -> NamedExternref("Int32Array")`.
        const ret = getArrayI32FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_bbb4883c6389f1de = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(U16)) -> NamedExternref("Uint16Array")`.
        const ret = getArrayU16FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_ca484d2c71e328c5 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 854, function: Function { arguments: [NamedExternref("TouchEvent")], shim_idx: 855, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_795bb83e9b24f476___closure__destroy___dyn_core_a2fb1ce8e50b7bd0___ops__function__FnMut__js_sys_fe2656c3903cbcb0___Array____Output_______, wasm_bindgen_795bb83e9b24f476___convert__closures_____invoke___js_sys_fe2656c3903cbcb0___Array_____);
        return ret;
    };
    imports.wbg.__wbindgen_cast_cb9088102bce6b30 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
        const ret = getArrayU8FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_cd07b1914aa3d62c = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
        const ret = getArrayF32FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
        // Cast intrinsic for `F64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_cast_e47ceb6027f5c92c = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(I16)) -> NamedExternref("Int16Array")`.
        const ret = getArrayI16FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_feefb5fadd6457fd = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(I8)) -> NamedExternref("Int8Array")`.
        const ret = getArrayI8FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedInt16ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedInt8ArrayMemory0 = null;
    cachedUint16ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('gpui_web_example_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
