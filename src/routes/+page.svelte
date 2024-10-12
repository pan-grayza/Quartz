<script lang="ts">
    import Button from 'src/components/Button.svelte'
    import { goto } from '$app/navigation'
</script>

<div>
    <p>Welcome</p>
    <p>
        Start by linking directory you want to share, or want to receive
        contents into
    </p>
    <Button
        onClick={() => {
            goto(`/link-directory`)
        }}>Go link directory</Button
    >
</div>
