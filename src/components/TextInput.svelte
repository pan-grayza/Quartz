<script lang="ts">
    export let value = ''
    export let className: string = ''
    export let placeholder: string = ''
    export let disabled: boolean = false
</script>

{#if !disabled}
    <input
        type="text"
        {placeholder}
        bind:value
        class={'px-2 py-1 relative bg-DarkModeTextInputBG placeholder:text-neutral-600' +
            ' ' +
            className}
        {...$$restProps}
    />
{:else}
    <input
        type="text"
        {placeholder}
        {value}
        class={'px-2 py-1 relative cursor-not-allowed bg-neutral-700 placeholder:text-neutral-500' +
            ' ' +
            className}
        disabled
        {...$$restProps}
    />
{/if}
