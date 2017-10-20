// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

/* global WebAssembly */

import wasmBuffer from './ethkey.wasm.js';

const NOOP = () => {};

// WASM memory setup
const WASM_PAGE_SIZE = 65536;
const STATIC_BASE = 1024;
const STATICTOP = STATIC_BASE + WASM_PAGE_SIZE * 2;
const STACK_BASE = align(STATICTOP + 16);
const STACKTOP = STACK_BASE;
const TOTAL_STACK = 5 * 1024 * 1024;
const TOTAL_MEMORY = 16777216;
const STACK_MAX = STACK_BASE + TOTAL_STACK;
const DYNAMIC_BASE = STACK_MAX + 64;
const DYNAMICTOP_PTR = STACK_MAX;

function mockWebAssembly () {
  function throwWasmError () {
    throw new Error('Missing WebAssembly support');
  }

  // Simple mock replacement
  return {
    Memory: class { buffer = new ArrayBuffer(2048) },
    Table: class {},
    Module: class {},
    Instance: class {
      exports = {
        '_input_ptr': () => 0,
        '_secret_ptr': () => 0,
        '_public_ptr': () => 0,
        '_address_ptr': () => 0,
        '_ecpointg': NOOP,
        '_brain': throwWasmError,
        '_verify_secret': throwWasmError
      }
    }
  };
}

const { Memory, Table, Module, Instance } = typeof WebAssembly !== 'undefined' ? WebAssembly : mockWebAssembly();

const wasmMemory = new Memory({
  initial: TOTAL_MEMORY / WASM_PAGE_SIZE,
  maximum: TOTAL_MEMORY / WASM_PAGE_SIZE
});

const wasmTable = new Table({
  initial: 8,
  maximum: 8,
  element: 'anyfunc'
});

// TypedArray views into the memory
const wasmMemoryU8 = new Uint8Array(wasmMemory.buffer);
const wasmMemoryU32 = new Uint32Array(wasmMemory.buffer);

// Keep DYNAMIC_BASE in memory
wasmMemoryU32[DYNAMICTOP_PTR >> 2] = align(DYNAMIC_BASE);

function align (mem) {
  const ALIGN_SIZE = 16;

  return (Math.ceil(mem / ALIGN_SIZE) * ALIGN_SIZE) | 0;
}

export function slice (ptr, len) {
  return wasmMemoryU8.subarray(ptr, ptr + len);
}

// Required by emscripten
function abort (what) {
  throw new Error(what || 'WASM abort');
}

// Required by emscripten
function abortOnCannotGrowMemory () {
  abort(`Cannot enlarge memory arrays.`);
}

// Required by emscripten
function enlargeMemory () {
  abortOnCannotGrowMemory();
}

// Required by emscripten
function getTotalMemory () {
  return TOTAL_MEMORY;
}

// Required by emscripten - used to perform memcpy on large data
function memcpy (dest, src, len) {
  wasmMemoryU8.set(wasmMemoryU8.subarray(src, src + len), dest);

  return dest;
}

// Synchronously compile WASM from the buffer
const module = new Module(wasmBuffer);

// Instantiated WASM module
const instance = new Instance(module, {
  global: {},
  env: {
    DYNAMICTOP_PTR,
    STACKTOP,
    STACK_MAX,
    abort,
    enlargeMemory,
    getTotalMemory,
    abortOnCannotGrowMemory,
    ___lock: NOOP,
    ___syscall6: () => 0,
    ___setErrNo: (no) => no,
    _abort: abort,
    ___syscall140: () => 0,
    _emscripten_memcpy_big: memcpy,
    ___syscall54: () => 0,
    ___unlock: NOOP,
    _llvm_trap: abort,
    ___syscall146: () => 0,
    'memory': wasmMemory,
    'table': wasmTable,
    tableBase: 0,
    memoryBase: STATIC_BASE
  }
});

export const extern = instance.exports;
