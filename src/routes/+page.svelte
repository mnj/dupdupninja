<script lang="ts">
  import '../styles/global.css';

  import MenuBar from "../lib/components/MenuBar.svelte";
  import Toolbar from "../lib/components/Toolbar.svelte";
  import SplitView from "../lib/components/SplitView.svelte";
  import StatusBar from "../lib/components/StatusBar.svelte";

  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
</script>

<div class="app">
  <header class="header">
    <MenuBar />
    <Toolbar />
  </header>

  <main class="main">
    <SplitView />
  </main>

  <footer class="footer">
    <StatusBar />
  </footer>
</div>

<style>
 :global(:root) {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 14px;
    line-height: 1.4;
    --header-height: auto; /* calculated by content */
    --toolbar-height: 48px;
    --statusbar-height: 26px;
    color: #0f0f0f;
    background: #f6f6f6;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden; /* critical: only inner panes scroll */
  }

  .header {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    user-select: none;
    background: #ffffff;
    border-bottom: 1px solid #dcdce0;
  }

  .main {
    flex: 1 1 auto;
    display: flex;
    overflow: hidden; /* ensures inner scrolls don't propagate */    
  }

  .footer {
    flex-shrink: 0;
    background: #f0f0f0;
    border-top: 1px solid #dcdce0;
  }

/* .logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
} */

</style>
