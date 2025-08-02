<script lang="ts">
  import { onDestroy } from "svelte";

  type MenuItem = { label: string; action: () => void };
  type Menu = { title: string; items: MenuItem[] };

  const menus: Menu[] = [
    {
      title: 'File',
      items: [
        { label: 'Save', action: () => console.log('Save triggered') },
        { label: 'Exit', action: () => window.close?.() },
      ],
    },
    {
      title: 'Scan',
      items: [
        { label: 'Folder', action: () => console.log('Scan Folder') },
        { label: 'Device', action: () => console.log('Scan Device') },
      ],
    },
  ];

  let openMenu: string | null = null;

  function toggle(menu: string) {
    openMenu = openMenu === menu ? null : menu;
    console.log('Toggled menu:', openMenu);
  }

  function closeAll() {
    openMenu = null;
  }

  const onBodyClick = (e: MouseEvent) => {
    if (openMenu && !(e.target as HTMLElement).closest('.menu-bar')) {
      closeAll();
    }
  };
  window.addEventListener('click', onBodyClick);
  onDestroy(() => window.removeEventListener('click', onBodyClick));
</script>

<nav class="menu-bar">
  {#each menus as menu}
    <div class="menu-group">
      <button
        class="menu-trigger"
        on:click|stopPropagation={() => toggle(menu.title)}
        aria-haspopup="true"
        aria-expanded={openMenu === menu.title}
      >
        {menu.title}
        <span class="caret">▾</span>
      </button>
      {#if openMenu === menu.title}
        <div class="submenu">
          {#each menu.items as item}
            <div class="submenu-item" on:click={() => { item.action(); openMenu = null; }}>
              {item.label}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</nav>

<style>
  .menu-bar {
    display: flex;
    padding: 0 8px;
    background: var(--menu-bg);
    color: var(--menu-color);
    font-size: 14px;
    gap: 4px;
    align-items: center;
    height: 32px;
  }

  .menu-group {
    position: relative;
  }

  .menu-trigger {
    background: transparent;
    border: none;
    padding: 6px 10px;
    color: #f0f0f0;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 500;
  }
  .menu-trigger:hover,
  .menu-trigger:focus {
    background: rgba(255, 255, 255, 0.1);
    outline: none;
  }

  .caret {
    font-size: 0.6em;
    line-height: 1;
  }

  .submenu {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--surface);
    color: var(--on-surface);
    border: 1px solid var(--border);
    min-width: 140px;
    box-shadow: 0 6px 18px rgba(0,0,0,0.15);
    z-index: 10;
    padding: 4px 0;
    border-radius: 4px;
  }

  .submenu-item {
    padding: 6px 12px;
    cursor: pointer;
    font-size: 13px;
    white-space: nowrap;
  }

  .submenu-item:hover {
    background: #f0f4fa;
  }
</style>