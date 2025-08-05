import { writable, type Writable } from "svelte/store";
import { path as infoPath, type PluginInfo } from "./api/info";
import { toast } from "svelte-sonner";
import { authHeader } from "./auth";

export const title = writable("unknown");

export const running = writable(false);

export const plgsInfo: Writable<PluginInfo[]> = writable([]);

export const currentPluginId: Writable<string | null> = writable(null);

export async function updateInfo() {
  const response = await fetch(infoPath, {
    headers: {
      ...authHeader(),
    },
  });
  if (response.ok) {
    plgsInfo.set(await response.json())
  }else{
    toast.error("插件信息获取失败",{
      description: "请检查网络连接或重试"
    })
  }
}
