import type {} from "./$types";
import { path, type Data } from "$lib/api/info";
import { authHeader, isLogin } from "$lib/auth";
import { toast } from "svelte-sonner";

export const ssr = false;
export const prerender = true;

export async function load({ fetch }): Promise<Data> {
  if (isLogin()) {
    console.log("验证登陆")
    const verify = await fetch("/auth/verify", {
      headers: {
        ...authHeader(),
      },
    });
    if (!verify.ok) {
      toast.error(`登陆失效, 请重新登录。`);
      localStorage.removeItem("token");
    }
  }
  const response = await fetch(path, {
    headers: {
      ...authHeader(),
    },
  });
  if (!response.ok) {
    toast.error(`HTTP error! status: ${response.status}`);
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  const isRegistered = await fetch("/auth/is_registered");
  if (!isRegistered.ok) {
    toast.error(`HTTP error! status: ${isRegistered.status}`);
    throw new Error(`HTTP error! status: ${isRegistered.status}`);
  }
  return {
    plgsInfo: await response.json(),
    isRegistered: await isRegistered.json(),
  };
}
