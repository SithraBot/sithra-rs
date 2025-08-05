<script lang="ts">
    import HouseIcon from "@lucide/svelte/icons/house";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Plus from "@lucide/svelte/icons/plus";
    import Trash from "@lucide/svelte/icons/trash";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Copy from "@lucide/svelte/icons/copy";
    import PenLine from "@lucide/svelte/icons/pen-line";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index";
    import * as Sidebar from "$lib/components/ui/sidebar/index";
    import type { PluginInfo } from "$lib/api/info";
    import { beforeNavigate } from "$app/navigation";
    import {} from "$lib/components/action-bar.svelte";
    import { onMount } from "svelte";
    import { Input } from "$lib/components/ui/input/index.js";
    const { pluginsInfo }: { pluginsInfo: PluginInfo[] } = $props();

    // Menu items.
    const mainItems = [
        {
            title: "主页",
            url: "/",
            icon: HouseIcon,
        },
    ];

    console.debug(pluginsInfo);
    const pluginItems = $derived(
        pluginsInfo.map((plugin) => ({
            title: plugin.id,
            id: plugin.id,
            url: `/plugin/${plugin.id}`,
            icon: Boxes,
        })),
    );

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

<Sidebar.Root variant="floating" class="pr-0">
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Sithra</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each mainItems as item (item.title)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton
                                isActive={item.url === currentPathName}
                            >
                                {#snippet child({ props })}
                                    <a href={item.url} {...props}>
                                        <item.icon />
                                        <span>{item.title}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Plugins</Sidebar.GroupLabel>
            <Sidebar.GroupAction title="添加插件">
                <Plus /> <span class="sr-only">添加插件</span>
            </Sidebar.GroupAction>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each pluginItems as item (item.id)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton
                                isActive={item.url === currentPathName}
                            >
                                {#snippet child({ props })}
                                    <a href={item.url} data-sveltekit-preload-data="tap" {...props}>
                                        <item.icon />
                                        <span>{item.title}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                            <DropdownMenu.Root>
                                <DropdownMenu.Trigger>
                                    {#snippet child({ props })}
                                        <Sidebar.MenuAction {...props}>
                                            <Ellipsis />
                                        </Sidebar.MenuAction>
                                    {/snippet}
                                </DropdownMenu.Trigger>
                                <DropdownMenu.Content
                                    side="right"
                                    align="start"
                                >
                                    <DropdownMenu.Item
                                        onclick={() => {
                                            window.dispatchEvent(
                                                new CustomEvent(
                                                    "action:delete",
                                                    {
                                                        detail: {
                                                            id: item.id,
                                                        },
                                                    },
                                                ),
                                            );
                                        }}
                                    >
                                        <Trash />
                                        <span>删除实例</span>
                                    </DropdownMenu.Item>
                                    <DropdownMenu.Item
                                        onclick={() => {
                                            window.dispatchEvent(
                                                new CustomEvent("action:copy", {
                                                    detail: {
                                                        id: item.id,
                                                    },
                                                }),
                                            );
                                        }}
                                    >
                                        <Copy />
                                        <span>创建副本</span>
                                    </DropdownMenu.Item>
                                    <DropdownMenu.Item>
                                        <PenLine />
                                        <span>重命名</span>
                                    </DropdownMenu.Item>
                                </DropdownMenu.Content>
                            </DropdownMenu.Root>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
</Sidebar.Root>
