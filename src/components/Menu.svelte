<script lang="ts">
    export let unlinkDirectory: (name: string) => void
    import { linked_paths } from 'src/store'
</script>

<div class="flex flex-col justify-between w-full h-full p-2">
    <div>
        <p>Linked vaults</p>
        <ul>
            {#if $linked_paths && $linked_paths.length !== 0}
                {#each $linked_paths as linked_path}
                    <li>
                        {linked_path.name}
                        <button
                            on:click={() => unlinkDirectory(linked_path.name)}
                        >
                            Remove
                        </button>
                    </li>
                {/each}
            {:else}
                <p>No paths are linked</p>
            {/if}
        </ul>
    </div>

    <div class="flex flex-col w-full p-2 mt-4">
        <a href="/help" class="relative w-full p-[2px]">Help</a>
        <a href="/link-directory" class="relative w-full p-[2px]"
            >Link directory</a
        >
        <a href="/my-networks" class="relative w-full p-[2px]">My Networks</a>
        <a href="/settings" class="relative w-full p-[2px]">Settings</a>
    </div>
</div>
