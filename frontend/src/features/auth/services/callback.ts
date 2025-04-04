import { BACKEND_HOST } from "@services";
import { AuthInfo } from "../types/AuthInfo";

export async function authCallback(code: string): Promise<AuthInfo> {
  const res = await fetch(
    `${BACKEND_HOST}/auth/callback?code=${code}`,
  );

  return await res.json();
}

