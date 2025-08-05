<script lang="ts">
    import { currentPluginId, plgsInfo, updateInfo } from "$lib/state.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import {} from "$lib/components/action-bar.svelte";
    import { on } from "svelte/events";
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import Copy from "@lucide/svelte/icons/copy";
    import { goto } from "$app/navigation";
    import {
        path as clonePluginPath,
        type ClonePlugin,
    } from "$lib/api/clone-plugin";
    import { authHeader } from "$lib/auth";

    const validInput = (value: string) => {
        const regex = /^[a-zA-Z][a-zA-Z0-9_-]*$/;
        return regex.test(value);
    };
    let cloneDialogOpen = $state(false);
    let cloneInputValue = $state("");
    let pluginId = $state("");
    const valid = $derived(
        $plgsInfo.find((s) => s.id == cloneInputValue) == undefined &&
            validInput(cloneInputValue),
    );
    async function clonePlugin() {
        try {
            const data: ClonePlugin = {
                id: pluginId,
                to: cloneInputValue,
            };
            const res = await fetch(clonePluginPath, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    ...authHeader()
                },
                body: JSON.stringify(data),
            });
            if (res.ok) {
                cloneDialogOpen = false;
                cloneInputValue = "";
                toast.success(`实例 [${pluginId}] 复制成功`);
                await updateInfo();
                if ($currentPluginId == pluginId) {
                    goto("/");
                }
            } else {
                toast.error(`复制实例 [${pluginId}] 失败`, {
                    description: await res.text(),
                });
            }
        } catch (err) {
            console.error(err);
            toast.error(`复制实例 [${pluginId}] 失败`, {
                description: err?.toString() ?? "未知错误",
            });
        }
    }
    onMount(() => {
        const handlers: (() => void)[] = [];
        handlers.push(
            on(window, "action:copy", async (event) => {
                console.debug(event);
                if (!!event.detail.id) {
                    pluginId = event.detail.id;
                    cloneDialogOpen = true;
                }
            }),
        );
        return () => {
            handlers.forEach((handler) => handler());
        };
    });
</script>

<Dialog.Root bind:open={cloneDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>复制实例 [{pluginId}]</Dialog.Title>
            <Dialog.Description class="leading-6">
                复制实例会将插件配置复制，<br class="sm:hidden" />新的插件实例
                ID 不能与已存在的实例 ID 重复。
                <br />
                主要用于插件多例创建。
                <br />
                请输在下方入复制后的实例 ID，<br class="sm:hidden" />实例 ID
                必须以字母开头，只能包含字母、数字和 "_", "-"
            </Dialog.Description>
        </Dialog.Header>
        <Input
            autocomplete="off"
            type="text"
            aria-invalid={!valid}
            id="name"
            bind:value={cloneInputValue}
        />
        <Dialog.Footer>
            <Button
                variant="default"
                disabled={!valid}
                onclick={clonePlugin}><Copy /> 复制</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
