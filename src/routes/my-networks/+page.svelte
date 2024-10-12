<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { listen } from '@tauri-apps/api/event'
    import { networks } from 'src/store'
    async function get_networks() {
        await invoke<Network[]>('read_private_networks')
            .then((updated_networks) => ($networks = updated_networks))
            .catch((e) => console.error(e))
    }
    async function remove_network(network: Network) {
        await invoke('remove_network', { network })
    }
    onMount(() => {
        listen('linked_paths_changed', async () => {
            await get_networks() // Re-fetch linked paths when the event is received
        })

        // Initial fetch of linked paths
        get_networks()
    })
</script>

<div class="relative flex flex-col w-full h-full">
    <div class="relative w-full h-full p-4">
        <p>Networks:</p>
        <ul>
            {#if $networks && $networks.length !== 0}
                {#each $networks as network}
                    <li>
                        {#if 'LocalNetwork' in network}
                            <p>{network.LocalNetwork.name}</p>
                        {:else if 'InternetNetwork' in network}
                            <p>{network.InternetNetwork.name}</p>
                        {:else if 'DarkWebNetwork' in network}
                            <p>{network.DarkWebNetwork.name}</p>
                        {/if}
                        <button on:click={() => remove_network(network)}>
                            Remove
                        </button>
                    </li>
                {/each}
            {:else}
                <p>No Networks</p>
            {/if}
        </ul>
    </div>
    <div class="relative flex flex-row items-center justify-center gap-16 p-4">
        <a href="/my-networks/create">Create Network</a>
        <a href="/my-networks/join">Join Network</a>
    </div>
</div>
