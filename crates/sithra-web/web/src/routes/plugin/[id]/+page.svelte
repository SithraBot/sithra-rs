<script lang="ts">
    import { currentPluginId, running, title } from "$lib/state.svelte";
    import { Separator } from "$lib/components/ui/separator/index";
    import { Badge } from "$lib/components/ui/badge/index";
    import Blocks from "@lucide/svelte/icons/blocks";
    import Folder from "@lucide/svelte/icons/folder";
    import { shortenPath } from "$lib/path-utils";
    import Editor from "$lib/toml/editor.svelte";
    import {} from "$lib/components/action-bar.svelte";
    import { on } from "svelte/events";
    import { onMount } from "svelte";
    import * as Toml from "@iarna/toml";
    import { toast } from "svelte-sonner";
    import { isParseError } from "$lib/toml/toml-utils.js";
    import {
        path as saveConfigPath,
        type SaveConfig,
    } from "$lib/api/save-config.js";
    import {
        path as ctrlPluginPath,
        type CtrlPlugin,
    } from "$lib/api/ctrl-plugin.js";
    import { path as delPluginPath } from "$lib/api/del-plugin.js";
    import { authHeader } from "$lib/auth.js";

    const { data } = $props();
    let editor: Editor;
    $effect(() => {
        title.set(data.details.id);
        running.set(data.details.running);
        console.debug(data.details);
        currentPluginId.set(data.details.id);
    });
    $inspect(data);
    onMount(() => {
        const handlers: (() => void)[] = [];
        handlers.push(
            on(window, "action:apply", async () => {
                const content = editor.getContent();
                try {
                    Toml.parse(content);
                } catch (err) {
                    if (isParseError(err)) {
                        toast.error(`Toml 解析发生错误`, {
                            description: `错误位置: ${err.line}行 ${err.col}列`,
                        });
                    } else {
                        toast.error(err?.toString() ?? "发生未知错误");
                    }
                    return;
                }
                const body: SaveConfig = {
                    id: data.details.id,
                    config: content,
                };
                await request(
                    body,
                    saveConfigPath,
                    `插件 [${data.details.id}] 配置应用成功`,
                    `插件 [${data.details.id}] 配置应用失败`,
                );
            }),
        );
        handlers.push(
            on(window, "action:start", async () => {
                const enable = !$running;
                const body: CtrlPlugin = { id: data.details.id, enable };
                if (
                    await request(
                        body,
                        ctrlPluginPath,
                        `插件 [${data.details.id}] ${enable ? "启动" : "停止"}成功`,
                        `插件 [${data.details.id}] ${enable ? "启动" : "停止"}失败`,
                    )
                ) {
                    running.set(enable);
                }
            }),
        );
        return () => {
            handlers.forEach((handler) => handler());
        };
    });

    async function request(
        body: any,
        path: string,
        sucessMessage: string,
        errorMessage: string,
    ): Promise<boolean> {
        try {
            const res = await fetch(path, {
                method: "POST",
                body: JSON.stringify(body),
                headers: {
                    "Content-Type": "application/json",
                    ...authHeader(),
                },
            });
            if (res.ok) {
                toast.success(sucessMessage);
                return true;
            } else {
                toast.error(errorMessage, {
                    description: await res.text(),
                });
                return false;
            }
        } catch (err) {
            console.error(err);
            toast.error(`API请求失败 [${data.details.id}]`, {
                description: err?.toString() ?? "发生未知错误",
            });
            return false;
        }
    }
</script>

<div class="w-full h-full overflow-auto p-4 flex flex-col flex-nowrap">
    <div
        class="flex flex-col justify-start items-baseline gap-2 lg:flex-row lg:gap-4 shrink-0"
    >
        <h1 class="text-2xl text-foreground">
            {data.details.name}
        </h1>
        <Badge variant="secondary">
            <Blocks />{data.details.version}
        </Badge>
        <Badge variant="secondary">
            <Folder /><span class="truncate"
                >{shortenPath(data.details.path, 30)}</span
            >
        </Badge>
    </div>
    <Separator class="my-4" />
    <Editor
        class="min-h-0 flex-1"
        default={data.details.toml_str}
        bind:this={editor}
    />
</div>
