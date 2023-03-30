import init from './lib/dslcad_server.js'
import { setServer } from "./env.js";

export async function run(server) {
    let module;
    setServer((len, ptr, cb) => {
        const input = new Int8Array(module.memory.buffer, ptr, len);
        const res = server.send(input);

        const buffer = module.new_buffer(res.length);

        const array = new Int8Array(module.memory.buffer, buffer, res.length);
        array.set(res);

        module.__wbindgen_export_2.get(cb)(array.length, array.byteOffset);

        module.drop_buffer(array.length, array.byteOffset);
    });
    const { default: init } = await import("./lib/dslcad.js");
     module = await init();
     module.main();
}

export function newServer() {
    return new Promise(resolve => {
        init({
            instantiateWasm: async function (imports, successCallback) {
                WebAssembly.instantiateStreaming(fetch("./lib/dslcad_wasm_server.wasm"), imports).then(
                    ({instance, module}) => {
                        successCallback(instance, module);
                    }
                );
            },
            onRuntimeInitialized: function () {
                let message;
                const cb = this.addFunction((length, buffer) => {
                    const tmp = this.HEAP8.subarray(buffer, buffer + length);
                    message = Int8Array.from(tmp);
                }, "vii");

                resolve({
                    send: (data) => {
                        let buffer = this._new_buffer(data.length);
                        const array = this.HEAP8.subarray(buffer, buffer + data.length);
                        array.set(data);
                        this._server(array.length, array.byteOffset, cb);

                        this._drop_buffer(array.length, array.byteOffset);
                        return message;
                    }
                })
            }
        });
    })
}