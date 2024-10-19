<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { listen } from '@tauri-apps/api/event'
    import { networks } from 'src/store'

    async function getNetworks() {
        await invoke<Network[]>('read_private_networks')
            .then((updated_networks) => ($networks = updated_networks))
            .catch((e) => console.error(e))
    }

    onMount(() => {
        listen('linked_paths_changed', async () => {
            await getNetworks()
        })

        getNetworks()
    })
    async function removeNetwork(networkName: string) {
        await invoke('remove_network', { networkName: 'Home' })
    }
</script>

<div class="relative flex flex-col w-full h-full">
    <div class="relative w-full h-full p-4">
        <p>Networks:</p>
        <ul>
            {#if $networks && $networks.length !== 0}
                {#each $networks as network}
                    <li>
                        <a href={`my-networks/${network.name}`}
                            >{network.name}</a
                        >

                        <button on:click={() => removeNetwork(network.name)}>
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
