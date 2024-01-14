<style>
    .layout {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1em;
    }
    
    .display {
        height: 512px;
        width: 100%;
        display: block;
    }
    
    .editor {
        width: 100%;
        display: flex;
        align-items: end;
        gap: 1em;
    }
    
    .editor textarea {
        width: 100%;
        min-height: 5em;
    }
</style>

<div class="layout">
    <div class="editor">
        <textarea id="input" rows="5">cube();</textarea>
        <button id="render" class="md-button">Render</button>
    </div>
    <div class="display">
        <canvas id="dslcad"></canvas>
    </div>
</div>

<script type="module">
    import preview from "./preview.js";
    import dslcad from "./dslcad.js";

    let btn = document.getElementById("render");
    let editor = document.getElementById("input");
    let module = await preview();

    const urlParams = new URLSearchParams(window.location.search);
    const source = urlParams.get('source');
    if (source) {
        editor.value = source;
        setTimeout(() => render(editor.value), 100);
    }


    editor.onchange = (event) => {
        if (history.pushState) {
            const newurl = window.location.protocol + "//" + window.location.host + window.location.pathname + `?source=${encodeURIComponent(editor.value)}`;
            window.history.pushState({path:newurl},'',newurl);
        }
    };

    function copyBufferTo(module, data) {
        let len = data.length;
        let ptr = module.allocate(len);
        const input = new Int8Array(module.memory.buffer, ptr, len);
        input.set(data);
        return [ptr, len];
    }

    document.addEventListener('keydown', e => {
        if (e.ctrlKey && e.key === 's') {
            e.preventDefault();
            render(editor.value);
        }
    });

    async function render(text) {
        module.show_rendering();

        let errorBuffer;
        try {

            let cad = await dslcad({ noInitialRun: true, printErr: (text) => { errorBuffer += text }});
            cad.FS.writeFile("input.ds", text);
            cad.callMain(["input.ds", "-o", "raw"]);

            const out = cad.FS.readFile("input.bin");
            const [ptr, len] = copyBufferTo(module, out);

            module.show_render(ptr, len);
        } catch (e) {
            const encoder = new TextEncoder();

            const error = encoder.encode(errorBuffer ?? e.toString());
            const [ptr, len] = copyBufferTo(module, error);

            module.show_error(ptr, len);
        }
    }


    btn.onclick = async () => {
        btn.disabled = true;
        await render(editor.value);
        btn.disabled = false;
    };

    module.main(0, 0);
</script>