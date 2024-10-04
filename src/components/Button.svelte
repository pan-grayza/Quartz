<script lang="ts">
    export let className = ''
    export let onClick = () => {}
    export let styleType: 'primary' | 'secondary' | 'tertiary' | 'danger' =
        'primary'
    $: buttonClass =
        styleType === 'primary'
            ? 'px-2 py-1 bg-DarkModePrimBtn hover:bg-DarkModePrimBtnHover text-DarkModePrimBtnText'
            : styleType === 'secondary'
              ? 'px-2 py-1 bg-DarkModeSecBtn hover:bg-DarkModeSecBtnHover text-DarkModeSecBtnText'
              : styleType === 'tertiary'
                ? 'px-2 py-1 bg-DarkModeTriBtn hover:bg-DarkModeTriBtnHover text-DarkModeTriBtnText'
                : 'px-2 py-1 bg-DarkModeDangerBtn text-white hover:bg-DarkModeDangerBtnHover text-DarkModeDangerBtnText'
</script>

<button
    class={buttonClass + ' ' + className}
    on:mouseup={onClick}
    {...$$restProps}><slot /></button
>
