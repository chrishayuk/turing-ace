// A minimal WASI (preview 1) host for Node: enough for the ace program,
// which only reads standard input and writes standard output and standard
// error. It needs no other library.
"use strict";

const fs = require("fs");

const ERRNO_BADF = 8;
const ERRNO_NOENT = 44;
const FILETYPE_CHARACTER_DEVICE = 2;

const pause = new Int32Array(new SharedArrayBuffer(4));
const sleep = (ms) => Atomics.wait(pause, 0, 0, ms);

// Standard input may be a pipe that is not ready yet: wait and try again.
function readInput(buffer) {
  for (;;) {
    try {
      return fs.readSync(0, buffer, 0, buffer.length, null);
    } catch (error) {
      if (error.code === "EOF") return 0;
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

function writeOutput(fd, bytes) {
  let done = 0;
  while (done < bytes.length) {
    try {
      done += fs.writeSync(fd, bytes, done, bytes.length - done);
    } catch (error) {
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

/**
 * Runs a WASI command to completion and returns its exit code.
 *
 * @param {string} wasmPath   the compiled program
 * @param {object} [options]
 * @param {string[]} [options.args]      arguments after the program name
 * @param {Uint8Array} [options.prefix]  bytes the program reads before
 *        anything from the real standard input
 */
function run(wasmPath, { args = [], prefix = new Uint8Array(0) } = {}) {
  const module = new WebAssembly.Module(fs.readFileSync(wasmPath));
  let memory;
  let position = 0;
  const view = () => new DataView(memory.buffer);
  const bytes = () => new Uint8Array(memory.buffer);
  const argv = ["ace", ...args].map((arg) => Buffer.from(arg + "\0"));

  const wasi = {
    args_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, argv.length, true);
      view().setUint32(sizePtr, argv.reduce((n, a) => n + a.length, 0), true);
      return 0;
    },
    args_get(argvPtr, bufPtr) {
      for (const arg of argv) {
        view().setUint32(argvPtr, bufPtr, true);
        bytes().set(arg, bufPtr);
        argvPtr += 4;
        bufPtr += arg.length;
      }
      return 0;
    },
    environ_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, 0, true);
      view().setUint32(sizePtr, 0, true);
      return 0;
    },
    environ_get() {
      return 0;
    },
    random_get(ptr, length) {
      require("crypto").randomFillSync(bytes(), ptr, length);
      return 0;
    },
    fd_read(fd, iovsPtr, iovsLen, nreadPtr) {
      if (fd !== 0) return ERRNO_BADF;
      let total = 0;
      for (let i = 0; i < iovsLen && total === 0; i++) {
        const ptr = view().getUint32(iovsPtr + i * 8, true);
        const len = view().getUint32(iovsPtr + i * 8 + 4, true);
        if (len === 0) continue;
        if (position < prefix.length) {
          const chunk = prefix.subarray(position, position + len);
          bytes().set(chunk, ptr);
          position += chunk.length;
          total = chunk.length;
        } else {
          const buffer = Buffer.alloc(len);
          total = readInput(buffer);
          bytes().set(buffer.subarray(0, total), ptr);
          if (total === 0) break;
        }
      }
      view().setUint32(nreadPtr, total, true);
      return 0;
    },
    fd_write(fd, iovsPtr, iovsLen, nwrittenPtr) {
      if (fd !== 1 && fd !== 2) return ERRNO_BADF;
      let total = 0;
      for (let i = 0; i < iovsLen; i++) {
        const ptr = view().getUint32(iovsPtr + i * 8, true);
        const len = view().getUint32(iovsPtr + i * 8 + 4, true);
        if (len > 0) writeOutput(fd, bytes().slice(ptr, ptr + len));
        total += len;
      }
      view().setUint32(nwrittenPtr, total, true);
      return 0;
    },
    fd_fdstat_get(fd, statPtr) {
      if (fd > 2) return ERRNO_BADF;
      bytes().fill(0, statPtr, statPtr + 24);
      view().setUint8(statPtr, FILETYPE_CHARACTER_DEVICE);
      return 0;
    },
    // Asked of standard input to see whether it is a terminal. It is not.
    fd_filestat_get(fd, statPtr) {
      if (fd > 2) return ERRNO_BADF;
      bytes().fill(0, statPtr, statPtr + 64);
      view().setUint8(statPtr + 16, FILETYPE_CHARACTER_DEVICE);
      return 0;
    },
    fd_close() {
      return 0;
    },
    // No files: there are no preopened directories and nothing can be opened.
    fd_prestat_get() {
      return ERRNO_BADF;
    },
    fd_prestat_dir_name() {
      return ERRNO_BADF;
    },
    path_open() {
      return ERRNO_NOENT;
    },
    proc_exit(code) {
      throw new Exit(code);
    },
  };

  const instance = new WebAssembly.Instance(module, { wasi_snapshot_preview1: wasi });
  memory = instance.exports.memory;
  try {
    instance.exports._start();
    return 0;
  } catch (error) {
    if (error instanceof Exit) return error.code;
    throw error;
  }
}

class Exit extends Error {
  constructor(code) {
    super(`exit ${code}`);
    this.code = code;
  }
}

module.exports = { run };
