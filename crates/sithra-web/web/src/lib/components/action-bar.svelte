<script lang="ts">
    import { Button } from "./ui/button/index";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import Play from "@lucide/svelte/icons/play";
    import Trash from "@lucide/svelte/icons/trash";
    import Copy from "@lucide/svelte/icons/copy";
    import Check from "@lucide/svelte/icons/check";
    const { disabled }: { disabled?: boolean } = $props();
</script>

<div
    class="bg-secondary p-1 flex items-center border shadow-sm rounded-lg gap-2"
>
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"><Play /></Button
    >
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"><Check /></Button
    >
    <Separator
        orientation="vertical"
        class="mx-1 data-[orientation=vertical]:h-2"
    />
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"><Trash /></Button
    >
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"><Copy /></Button
    >
</div>
