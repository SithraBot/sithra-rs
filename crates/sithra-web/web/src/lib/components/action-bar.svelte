<script module lang="ts">
    declare global {
        interface WindowEventMap {
            "action:start": CustomEvent<undefined>;
            "action:apply": CustomEvent<undefined>;
            "action:delete": CustomEvent<{ id?: string }>;
            "action:copy": CustomEvent<{ id?: string }>;
        }
    }
</script>

<script lang="ts">
    import { Button } from "./ui/button/index";
    import { Separator } from "$lib/components/ui/separator/index";
    import Square from "@lucide/svelte/icons/square";
    import Play from "@lucide/svelte/icons/play";
    import Trash from "@lucide/svelte/icons/trash";
    import Copy from "@lucide/svelte/icons/copy";
    import Check from "@lucide/svelte/icons/check";
    import { onMount } from "svelte";
    import { on } from "svelte/events";
    import { currentPluginId, running } from "$lib/state.svelte";
    const { disabled }: { disabled?: boolean } = $props();
    function dispatch(event: string, detail?: any) {
        window.dispatchEvent(new CustomEvent(event, { detail }));
    }
    onMount(() => {
        const handlers: (() => void)[] = [];
        handlers.push(
            on(window, "keypress", (event) => {
                if ((event.ctrlKey || event.metaKey) && event.key == "s") {
                    event.preventDefault();
                    dispatch("apply");
                }
            }),
        );
        return () => {
            handlers.forEach((handler) => handler());
        };
    });
</script>

<div
    class="bg-secondary/50 p-1 flex items-center border shadow-sm rounded-lg gap-2 mb-0.5"
>
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"
        onclick={() => dispatch("action:start")}
    >
        <Play
            class="h-[1.2rem] w-[1.2rem] scale-100 !transition-all [&[data-running=true]]:scale-0"
            data-running={$running}
        />
        <Square
            class="absolute h-[1.2rem] w-[1.2rem] scale-0 !transition-all [&[data-running=true]]:scale-100"
            data-running={$running}
        />
    </Button>
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"
        onclick={() => dispatch("action:apply")}><Check /></Button
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
        variant="ghost"
        onclick={() => dispatch("action:delete", { id: $currentPluginId })}
        ><Trash /></Button
    >
    <Button
        {disabled}
        size="icon"
        class="size-6 ring-sidebar-ring active:bg-sidebar-accent
        hover:bg-sidebar-accent dark:hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        variant="ghost"
        onclick={() => dispatch("action:copy", { id: $currentPluginId })}
        ><Copy /></Button
    >
</div>
