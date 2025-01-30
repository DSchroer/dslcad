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
        flex-direction: column;
    }
    
    .editor textarea {
        width: 100%;
        min-height: 5em;
        margin-bottom: 0;
    }

    .editor .actions {
        display: flex;
        width: 100%;
        gap: 0.5em;
        justify-content: right;
    }
</style>

<div class="layout">
    <div class="editor">
        <textarea id="input" rows="5">cube();</textarea>
        <div class="actions">
            <button id="download" class="md-button">Download</button>
            <button id="render" class="md-button md-button--primary">Render</button>
        </div>
    </div>
    <div class="display">
        <canvas id="dslcad"></canvas>
    </div>
</div>

<script type="module">
    import preview from "./dslcad-viewer.js";
    import dslcad from "./dslcad.js";

    let renderBtn = document.getElementById("render");
    let downloadBtn = document.getElementById("download");
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

    async function render(text, download) {
        module.show_rendering();

        let errorBuffer;
        try {

            let cad = await dslcad({ noInitialRun: true, printErr: (text) => { errorBuffer += text }});
            cad.FS.writeFile("input.ds", text);
            cad.callMain(["input.ds", "-o", "raw"]);

            const out = cad.FS.readFile("input.bin");

            if (download) {
                const blob = new Blob([out], {type: "application/bin"});
                const link = document.createElement('a');
                link.href = window.URL.createObjectURL(blob);
                link.download = "render.bin";
                link.click();
            }

            const [ptr, len] = copyBufferTo(module, out);

            module.show_render(ptr, len);
        } catch (e) {
            console.warn(e);
            const encoder = new TextEncoder();

            const error = encoder.encode(errorBuffer ?? e.toString());
            const [ptr, len] = copyBufferTo(module, error);

            module.show_error(ptr, len);
        }
    }


    renderBtn.onclick = async () => {
        renderBtn.disabled = true;
        await render(editor.value, false);
        renderBtn.disabled = false;
    };

    downloadBtn.onclick = async () => {
        renderBtn.disabled = true;
        await render(editor.value, true);
        downloadBtn.disabled = false;
    };

    module.main(0, 0);
</script>