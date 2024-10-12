<script lang="ts">
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { linked_paths } from 'src/store'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { listen } from '@tauri-apps/api/event'
    import 'src/app.css'
    // Components
    import Menu from 'src/components/Menu.svelte'
    // Icons
    import MinimizeIcon from 'src/icons/line.svelte'
    import MaximizeIcon from 'src/icons/copy.svelte'
    import SquareIcon from 'src/icons/square.svelte'
    import CloseIcon from 'src/icons/close.svelte'

    let isMaximized = false
    let isMenuOpen = false

    async function get_linked_paths() {
        await invoke<LinkedPath[]>('get_linked_paths')
            .then((paths) => ($linked_paths = paths))
            .catch((e) => console.error(e))
    }
    async function unlink_directory(pathName: string) {
        await invoke('unlink_directory', { pathName })
    }

    // Titlebar functionality
    const appWindow = getCurrentWindow()
    const checkWindowState = async () => {
        isMaximized = await appWindow.isMaximized()
    }
    function setupEventListeners() {
        appWindow.listen('tauri://resize', checkWindowState)
        appWindow.listen('tauri://move', checkWindowState)

        // Check the initial state
        checkWindowState()
    }
    onMount(() => {
        setupEventListeners()
        listen('linked_paths_changed', async () => {
            await get_linked_paths() // Re-fetch linked paths when the event is received
        })

        // Initial fetch of linked paths
        get_linked_paths()
    })
    function isMobileDevice() {
        return /Mobi|Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
            navigator.userAgent
        )
    }
</script>

<div class="flex flex-col w-full h-full {!isMobileDevice() && 'pt-[30px]'}">
    {#if !isMobileDevice()}
        <div data-tauri-drag-region class="w-screen titlebar">
            <p
                class="absolute translate-x-[-50%] pointer-events-none left-1/2 translate-y-[-50%] top-1/2"
            >
                Quartz
            </p>
            <button
                on:click={() => {
                    appWindow.minimize()
                }}
                class="titlebar-button"
                id="titlebar-minimize"
            >
                <MinimizeIcon />
            </button>
            <button
                on:click={() => {
                    appWindow.toggleMaximize()
                }}
                class="titlebar-button"
                id="titlebar-maximize"
            >
                {#if isMaximized}
                    <MaximizeIcon
                        size="14"
                        className="scale-x-[-1] scale-y-1"
                    />
                {:else}
                    <SquareIcon size="12" className="scale-x-[-1] scale-y-1" />
                {/if}
            </button>
            <button
                on:click={() => {
                    appWindow.close()
                }}
                class="titlebar-button titlebar-button-close"
                id="titlebar-close"
            >
                <CloseIcon size="19" className="" />
            </button>
        </div>
    {:else}
        <div class="w-screen h-2"></div>
    {/if}

    <div class="flex flex-row w-full h-full">
        <!-- Sidebar for desktop -->
        <nav
            class="relative flex-col hidden w-40 h-full border-r shrink-0 sm:flex border-neutral-700"
        >
            <Menu {unlink_directory} />
        </nav>

        <!-- Main content -->
        <slot />

        <!-- Bottom navigation bar for mobile -->
        <div
            class="fixed bottom-0 flex items-center justify-center w-full p-4 bg-neutral-800 sm:hidden"
        >
            <button
                class="px-4 py-2 text-white bg-blue-500 rounded"
                on:click={() => (isMenuOpen = !isMenuOpen)}
            >
                Open Menu
            </button>
        </div>

        <!-- Nav menu for smaller screens -->
        {#if isMenuOpen}
            <div
                class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 sm:hidden"
            >
                <button
                    class="absolute w-full h-full bg-transparent border-none outline-none"
                    on:click={() => (isMenuOpen = false)}
                >
                </button>
                <div class="z-10 w-3/4 p-4 overflow-auto h-3/4">
                    <Menu {unlink_directory} />
                </div>
            </div>
        {/if}
    </div>
</div>
