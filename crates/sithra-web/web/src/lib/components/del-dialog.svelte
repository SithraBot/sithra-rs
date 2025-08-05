<script lang="ts">
    import { currentPluginId, updateInfo } from "$lib/state.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import {} from "$lib/components/action-bar.svelte";
    import { on } from "svelte/events";
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { path as delPluginPath } from "$lib/api/del-plugin.js";
    import Trash from "@lucide/svelte/icons/trash";
    import { goto } from "$app/navigation";
    import { authHeader } from "$lib/auth";

    let delDialogOpen = $state(false);
    let delInputValue = $state("");
    let pluginId = $state("");
    async function delPlugin() {
        try {
            const res = await fetch(delPluginPath + `/${pluginId}`, {
                method: "DELETE",
                headers: {
                    ...authHeader(),
                },
            });
            if (res.ok) {
                delDialogOpen = false;
                delInputValue = "";
                toast.success(`实例 [${pluginId}] 已删除`);
                await updateInfo();
                if ($currentPluginId == pluginId) {
                    goto("/");
                }
            } else {
                toast.error(`删除实例 [${pluginId}] 失败`, {
                    description: await res.text(),
                });
            }
        } catch (err) {
            console.error(err);
            toast.error(`删除实例 [${pluginId}] 失败`, {
                description: err?.toString() ?? "未知错误",
            });
        }
    }
    onMount(() => {
        const handlers: (() => void)[] = [];
        handlers.push(
            on(window, "action:delete", async (event) => {
                console.debug(event);
                if (!!event.detail.id) {
                    pluginId = event.detail.id;
                    delDialogOpen = true;
                }
            }),
        );
        return () => {
            handlers.forEach((handler) => handler());
        };
    });
</script>

<Dialog.Root bind:open={delDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>删除实例 [{pluginId}]</Dialog.Title>
            <Dialog.Description class="leading-6">
                删除实例不意味着删除插件，<br
                    class="sm:hidden"
                />插件本体将会被保留。
                <br />
                删除实例意味着配置将被删除，<br
                    class="sm:hidden"
                />配置删除后将无法找回。
                <br />
                请确认您要删除此插件实例。
                <br />
                欲确认删除，请在下方输入 "delete/{pluginId}"
            </Dialog.Description>
        </Dialog.Header>
        <Input
            autocomplete="off"
            type="text"
            aria-invalid={delInputValue !== `delete/${pluginId}`}
            id="name"
            bind:value={delInputValue}
            placeholder="delete/{pluginId}"
        />
        <Dialog.Footer>
            <Button
                variant="destructive"
                disabled={delInputValue !== `delete/${pluginId}`}
                onclick={delPlugin}><Trash /> 删除</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
