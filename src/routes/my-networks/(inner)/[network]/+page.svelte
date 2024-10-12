<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    // Import the $page store from SvelteKit
    import { page } from '$app/stores'
    let network = $page.params.network
</script>

<div>
    <h1>Current Network: {network}</h1>
</div>
