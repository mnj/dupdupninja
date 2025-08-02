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

</style>
