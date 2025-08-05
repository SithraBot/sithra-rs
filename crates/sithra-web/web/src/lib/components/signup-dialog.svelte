<script lang="ts">
    import { encodeCredentials } from "$lib/auth";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { toast } from "svelte-sonner";
    let open = $state(true);
    function getOpen() {
        return open;
    }
    function setOpen(value: boolean) {}
    async function handleInputKeydown(event: KeyboardEvent) {
        if (event.key === "Enter") {
            await onsubmit();
        }
    }
    let password = $state("");
    let username = $state("");
    async function onsubmit() {
        const credentials = await encodeCredentials(username, password);
        if (!credentials) {
            toast.error("用户名或密码不能为空");
            return;
        }
        try {
            let req = await fetch("/auth/register", {
                method: "POST",
                body: JSON.stringify({ hex: credentials }),
                headers: {
                    "Content-Type": "application/json",
                },
            });
            if (!req.ok) {
                toast.error("授权失败", {
                    description: await req.text(),
                });
                return;
            }
            const data = await req.text();
            localStorage.setItem("token", data);
            toast.success("注册成功");
            open = false;
            setOpen(false);
        } catch (e) {
            toast.error("注册错误", {
                description: e?.toString() ?? "未知错误",
            });
        }
    }
</script>

<Dialog.Root bind:open={getOpen, setOpen}>
    <Dialog.Content
        class="sm:max-w-sm"
        escapeKeydownBehavior="ignore"
        interactOutsideBehavior="ignore"
    >
        <Dialog.Header>
            <Dialog.Title>设置账户</Dialog.Title>
            <Dialog.Description
                >请使设置后续登陆使用的账户密码。</Dialog.Description
            >
        </Dialog.Header>
        <form class="mt-5" {onsubmit}>
            <div class="flex flex-col gap-6">
                <div class="grid gap-2">
                    <Label for="name">用户名</Label>
                    <Input
                        id="username"
                        type="text"
                        autocomplete="username"
                        required
                        bind:value={username}
                        aria-invalid={username.trim() == ""}
                        onkeydown={handleInputKeydown}
                    />
                </div>
                <div class="grid gap-2">
                    <Label for="password">密码</Label>
                    <Input
                        id="password"
                        type="password"
                        autocomplete="current-password"
                        required
                        bind:value={password}
                        aria-invalid={password == ""}
                        onkeydown={handleInputKeydown}
                    />
                </div>
            </div>
        </form>
        <Dialog.Footer>
            <Button
                type="submit"
                class="w-full"
                onclick={onsubmit}
                disabled={username.trim() == "" || password == ""}>注册</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
