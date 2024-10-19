<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { linked_paths } from 'src/store'
    import Button from 'src/components/Button.svelte'
    import TextInput from 'src/components/TextInput.svelte'

    let serverStatus = ''
    let selected_linked_paths: LinkedPath[] = []
    let serverModeState: ServerMode = 'LocalHost'
    const serverModes: ServerMode[] = ['LocalHost', 'Internet', 'DarkWeb']
    let serverName = ''

    async function handleStartServer() {
        serverStatus = await invoke('start_file_server_command', {
            serverMode: serverModeState,
            linkedPaths: selected_linked_paths,
        })
    }
    async function handleCreateNetwork() {
        serverStatus = await invoke('create_local_network', {
            name: serverName,
            serverMode: serverModeState,
            linkedPaths: selected_linked_paths,
        })
    }
    async function handleStopServer() {
        await invoke('stop_file_server_command')
    }
    function handleCheckLinkedPaths(e: Event, linked_path: LinkedPath) {
        const checkbox = e.target as HTMLInputElement
        if (checkbox.checked) {
            selected_linked_paths = [...selected_linked_paths, linked_path]
        } else {
            selected_linked_paths = selected_linked_paths.filter(
                (p) => p.name != linked_path.name
            )
        }
    }
    function handleRadioServerModes(e: Event, serverMode: ServerMode) {
        const checkbox = e.target as HTMLInputElement
        if (checkbox.checked) {
            serverModeState = serverMode
        }
    }
</script>

<div class="relative flex flex-row w-full h-full gap-2">
    <div>
        <div>
            <h1>Choose server mode</h1>
            {#each serverModes as serverMode}
                <input
                    name={serverMode}
                    type="radio"
                    on:change={(e) => {
                        handleRadioServerModes(e, serverMode)
                    }}
                />

                <label for={serverMode}>{serverMode}</label>
            {/each}
        </div>

        <form class="flex flex-col p-2" on:submit={handleCreateNetwork}>
            {#if serverModeState === 'LocalHost'}
                <div class="flex flex-col gap-2">
                    <p>Name</p>
                    <TextInput
                        bind:value={serverName}
                        type="text"
                        placeholder="Name"
                    />
                    <p>Port (optional)</p>
                    <TextInput type="text" placeholder="1234" />
                    <p>Password (optional)</p>
                    <TextInput type="text" placeholder="11111111" />
                    <Button type="submit">Create local network</Button>
                </div>
            {:else if serverModeState === 'Internet'}
                <div class="flex flex-col gap-2">
                    <p>Name</p>
                    <TextInput type="text" placeholder="Name" />
                    <p>Port</p>
                    <TextInput type="text" placeholder="1234" />
                    Password
                    <TextInput type="text" placeholder="11111111" />
                </div>

                <Button>Coming soon</Button>
            {:else if serverModeState === 'DarkWeb'}
                <div class="flex flex-col gap-2">
                    <p>Name</p>
                    <TextInput type="text" placeholder="Name" />
                    <p>Port</p>
                    <TextInput type="text" placeholder="1234" />
                    Password
                    <TextInput type="text" placeholder="11111111" />
                </div>

                <Button>Coming soon</Button>
            {/if}
            {serverStatus}
        </form>
    </div>
    <div>
        <p>Select linked Paths you want to share:</p>
        <ul class="flex flex-wrap gap-2">
            {#if $linked_paths && $linked_paths.length !== 0}
                {#each $linked_paths as linked_path}
                    <li
                        class="relative flex flex-row items-center justify-center w-fit h-fit"
                    >
                        <input
                            id={linked_path.name}
                            name={linked_path.name}
                            on:change={(e) =>
                                handleCheckLinkedPaths(e, linked_path)}
                            type="checkbox"
                            class="absolute w-4 h-4 left-2"
                        />
                        <label for={linked_path.name}>
                            <p
                                class="py-1 pr-2 rounded-md pointer-events-none select-none pl-7 bg-zinc-400/25"
                            >
                                {linked_path.name}
                            </p>
                        </label>
                    </li>
                {/each}
            {:else}
                <p>No paths are linked</p>
            {/if}
        </ul>
    </div>
</div>
