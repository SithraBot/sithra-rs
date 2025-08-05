import { Sha256 } from "@aws-crypto/sha256-browser";
import { toast } from "svelte-sonner";

function toHex(uint8: Uint8Array): string {
  return Array.from(uint8)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
export async function encodeCredentials(
  username: string,
  password: string,
): Promise<string | void> {
  username = username.trim();
  if (username == "") {
    toast.error("用户名不能为空, 请输入用户名");
    return;
  }
  if (password == "") {
    toast.error("密码不能为空, 请输入密码");
    return;
  }
  const hash = new Sha256();
  hash.update(password, "utf8");
  const sha256first = await hash.digest();
  const hexFirst = toHex(sha256first);
  hash.reset();
  hash.update(username, "utf8");
  const sha256second = await hash.digest();
  const hexSecond = toHex(sha256second);
  const credentials = `${hexFirst};${hexSecond}`;
  return credentials;
}

export function isLogin(): boolean {
  return localStorage.getItem("token") != null;
}

export function authHeader(): Record<string, string> {
  const token = localStorage.getItem("token");
  if (!token) {
    return {};
  }
  console.debug("auth")
  return { Authorization: `Bearer ${token}` };
}
