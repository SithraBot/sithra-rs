<script lang="ts">
    import HouseIcon from "@lucide/svelte/icons/house";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Plus from "@lucide/svelte/icons/plus";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";

    // Menu items.
    const mainItems = [
        {
            title: "主页",
            url: "#",
            icon: HouseIcon,
        },
    ];

    const pluginItems = [
        {
            title: "插件1",
            url: "#",
            icon: Boxes,
        },
        {
            title: "插件2",
            url: "#",
            icon: Boxes,
        },
    ];
</script>

<Sidebar.Root variant="floating" class="pr-0">
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Sithra</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each mainItems as item (item.title)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton>
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
                    {#each pluginItems as item (item.title)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton>
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
    </Sidebar.Content>
</Sidebar.Root>
