<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  
  export let onSelect: (folder: string) => void = () => {};

  async function selectFolder() {
    const folder = await open({
      directory: true,
      multiple: false, // set to true if you want to allow multi-select
      title: 'Select a folder to scan'
    });
    if (folder && typeof folder === 'string') {
      // folder is a string with the path, or an array of paths if multiple:true
      onSelect(folder);
    }
  }
</script>

<button on:click={selectFolder}>
  Select Folder to Scan
</button>

