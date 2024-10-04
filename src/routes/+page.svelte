<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    // Components
    import TextInput from '../components/TextInput.svelte'
    import Button from '../components/Button.svelte'

    let newLinkedPathName = ''
    let newLinkedPathPath = ''
    let statusLinkedPath = ''

    async function select_directory() {
        try {
            // Invoke the Rust command and handle the response
            const result = await invoke<
                string | { Ok: string | null } | { Err: string }
            >('select_directory')

            if (typeof result === 'object' && result !== null) {
                if ('Ok' in result) {
                    if (result.Ok) {
                        newLinkedPathPath = result.Ok // Set the path if selection was successful
                        statusLinkedPath = 'Directory selected successfully!'
                    } else {
                        statusLinkedPath = 'No directory selected.'
                    }
                } else if ('Err' in result) {
                    statusLinkedPath = `Error: ${result.Err}` // Handle error case
                }
            } else if (typeof result === 'string') {
                // If a plain string (e.g., a path) is returned
                newLinkedPathPath = result
                statusLinkedPath = 'Directory selected successfully!'
            }
        } catch (error) {
            // Handle any potential invocation errors
            statusLinkedPath = `Error invoking select_directory: ${error}`
        }
    }

    async function link_directory() {
        if (typeof newLinkedPathName !== 'string') {
            statusLinkedPath = 'Invalid name'
            return
        }
        statusLinkedPath = await invoke('link_directory', {
            path: newLinkedPathPath,
            name: newLinkedPathName,
        })
    }
</script>

<div
    class="relative flex flex-col items-center justify-center w-full h-full p-2"
>
    <div class="relative flex flex-col gap-4 w-60 h-128">
        <h1 class="text-2xl">Link directory</h1>

        <form
            class="relative flex flex-col gap-2 text-lg"
            on:submit|preventDefault={() => {
                link_directory()
                newLinkedPathPath = ''
                newLinkedPathName = ''
            }}
        >
            <!-- Custom TextInput component doesn't work with bind:value -->
            <input
                type="text"
                placeholder="Enter a name..."
                class="relative px-2 py-1 rounded-md bg-neutral-900 placeholder:text-neutral-600"
                bind:value={newLinkedPathName}
            />

            <div class="relative flex flex-row w-full">
                <button
                    class="absolute z-10 w-full h-full"
                    on:click|preventDefault={() => select_directory()}
                />
                <TextInput
                    placeholder="Click to select Directory"
                    value={newLinkedPathPath}
                    disabled
                    className="w-full rounded-md "
                />
            </div>

            <p>{statusLinkedPath}</p>
            <Button className="rounded-md " type="submit">Link directory</Button
            >
        </form>
    </div>
</div>

<style>
</style>
