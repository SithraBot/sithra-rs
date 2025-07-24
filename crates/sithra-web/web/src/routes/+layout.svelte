<script lang="ts">
    import "../app.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index";
    import * as Breadcrumb from "$lib/components/ui/breadcrumb/index";
    import { Separator } from "$lib/components/ui/separator/index";
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import { ModeWatcher } from "mode-watcher";
    import DarkToggle from "$lib/components/dark-toggle.svelte";
    import ActionBar from "$lib/components/action-bar.svelte";

    let { children } = $props();
</script>

<ModeWatcher />
<Sidebar.Provider>
    <AppSidebar />
    <Sidebar.Inset>
        <header
            class="flex h-12 shrink-0 items-center justify-between px-2 pt-1"
        >
            <div class="flex items-center shrink-0 gap-2">
                <Sidebar.Trigger />
                <Separator
                    orientation="vertical"
                    class="mx-2 data-[orientation=vertical]:h-4"
                />
                <h1 class="font-bold">主页</h1>
            </div>

            <div class="flex items-center shrink-0 gap-2">
                <ActionBar />
                <Separator
                    orientation="vertical"
                    class="mx-2 data-[orientation=vertical]:h-4"
                />
                <DarkToggle />
            </div>
        </header>
        <div class="w-full h-full px-2 pb-2">
            <main class="border rounded-md w-full h-full"></main>
            {@render children?.()}
        </div>
    </Sidebar.Inset>
</Sidebar.Provider>
