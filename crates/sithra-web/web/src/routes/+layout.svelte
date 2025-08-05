<script lang="ts">
    import "../app.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index";
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import { ModeWatcher } from "mode-watcher";
    import { Toaster } from "$lib/components/ui/sonner/index";
    import Header from "$lib/components/header.svelte";
    import hotkeys from "hotkeys-js";
    import { plgsInfo } from "$lib/state.svelte";
    import DelDialog from "$lib/components/del-dialog.svelte";
    import CloneDialog from "$lib/components/clone-dialog.svelte";
    import LoginDialog from "$lib/components/login-dialog.svelte";
    import SignupDialog from "$lib/components/signup-dialog.svelte";
    import { isLogin } from "$lib/auth";
    let { children, data } = $props();

    hotkeys("ctrl+s", function (event) {
        event.preventDefault();
        window.dispatchEvent(new CustomEvent("action:apply"));
    });
    console.log("hotkeys registered");
    $effect(() => {
        plgsInfo.set(data.plgsInfo);
    });
    $inspect($plgsInfo);
</script>

<ModeWatcher />
<Toaster position="top-right" richColors closeButton />
<DelDialog />
<CloneDialog />
{#if !isLogin()}
    {#if data.isRegistered}
        <LoginDialog />
    {:else}
        <SignupDialog />
    {/if}
{/if}
<Sidebar.Provider class="h-full">
    <AppSidebar pluginsInfo={$plgsInfo} />
    <Sidebar.Inset class="overflow-auto h-full">
        <Header />
        <div class="w-full px-2 pb-2 h-[calc(100%-var(--spacing)*12)]">
            <div class="border rounded-md w-full h-full overflow-hidden">
                {@render children?.()}
            </div>
        </div>
    </Sidebar.Inset>
</Sidebar.Provider>
