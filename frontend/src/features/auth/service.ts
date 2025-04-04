import { setRedirectUrl } from "./callback";

export interface AuthInfo {
  jwt?: string;
  username?: string;
  name?: string;
  avatar_url?: string;
  uuid?: string;
}

export async function authCallback(code: string): Promise<AuthInfo> {
  const res = await fetch(
    `${import.meta.env.VITE_BACKEND_HOST}/auth/callback?code=${code}`,
  );

  return await res.json();
}

export async function loginGuest(guest_name: string): Promise<AuthInfo> {
  const res = await fetch(
    `${import.meta.env.VITE_BACKEND_HOST}/login-guest`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        guest_name,
      }),
    },
  );

  return await res.json();
}

export async function loginGithub() {
  setRedirectUrl();

  window.location.href = `${import.meta.env.VITE_BACKEND_HOST}/auth`
}
