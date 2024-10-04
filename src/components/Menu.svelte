<script lang="ts">
    export let linked_paths: void | LinkedPath[] = [] // Expect linked_paths to be passed as a prop
    export let unlink_directory: (name: string) => void // Expect unlink_directory function to be passed as a prop
</script>

<div class="flex flex-col justify-between w-full h-full p-2">
    <div>
        <p>Linked vaults</p>
        <ul>
            {#if linked_paths && linked_paths.length !== 0}
                {#each linked_paths as linked_path}
                    <li>
                        {linked_path.name}
                        <button
                            on:click={() => unlink_directory(linked_path.name)}
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
        <a href="/" class="block mb-2">Link directory</a>
        <a href="/network" class="block mb-2">Your Network</a>
        <a href="/settings" class="block">Settings</a>
    </div>
</div>
