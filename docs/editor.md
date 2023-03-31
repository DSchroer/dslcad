<style>
.md-sidebar--secondary {
  display: none !important;
}
</style>

<script type="importmap">
{
  "imports": {
    "env": "./env.js"
  }
}
</script>

<script type="module">
    import { newServer, run } from "./dslcad.js";

    const container = document.getElementById("dslcad-container");
    container.style.height = container.clientWidth * (screen.width / screen.height);

    const server = await newServer();
    await run(server);
</script>

To edit navigate to `Window > Editor`.

<div id="dslcad-container">
    <canvas id="dslcad"></canvas>
</div>

Note: Web editor is still experimental. Many features are missing and will be added back over time. 
