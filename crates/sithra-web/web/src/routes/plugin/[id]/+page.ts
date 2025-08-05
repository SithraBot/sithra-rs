import type {} from "./$types";
import { path, type PluginDetails } from "$lib/api/details";
import { toast } from "svelte-sonner";
import { authHeader } from "$lib/auth";

export const ssr = false;
export const prerender = "auto";

export async function load({
  fetch,
  params,
}): Promise<{ details: PluginDetails }> {
  const response = await fetch(`${path}/${params.id}`, {
    headers: {
      ...authHeader(),
    },
  });
  if (!response.ok) {
    toast.error(`获取实例信息失败 [${response.status}]`, {
      description: await response.text(),
    });
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  return { details: await response.json() };
}
