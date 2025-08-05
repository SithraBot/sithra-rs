<script lang="ts">
    import DarkToggle from "$lib/components/dark-toggle.svelte";
    import ActionBar from "$lib/components/action-bar.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar/index";
    import { Separator } from "$lib/components/ui/separator/index";
    import { title } from "$lib/state.svelte";
    import { onMount } from "svelte";
    import { beforeNavigate } from "$app/navigation";
    
    let currentPathName = $state("/");
    onMount(() => {
        currentPathName = window.location.pathname;
    });
    beforeNavigate((navigation) => {
        console.debug(navigation);
        let pathname = navigation.to?.url.pathname;
        if (pathname) {
            currentPathName = pathname;
        }
    });
</script>

<header class="flex h-12 shrink-0 items-center justify-between px-2 pt-1">
    <div class="flex items-center shrink-0 gap-2">
        <Sidebar.Trigger />
        <Separator
            orientation="vertical"
            class="mx-2 data-[orientation=vertical]:h-4"
        />
        <h1 class="font-bold">{$title}</h1>
    </div>

    <div class="flex items-center shrink-0 gap-2">
        <ActionBar disabled={currentPathName === "/"} />
        <Separator
            orientation="vertical"
            class="mx-2 data-[orientation=vertical]:h-4"
        />
        <DarkToggle />
    </div>
</header>
