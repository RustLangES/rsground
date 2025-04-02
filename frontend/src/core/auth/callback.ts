import { batch } from "solid-js";
import { setAuthInfo, setIsLoadingAuthInfo } from "./store";

const REDIRECT_KEY = "redirect_url";

export function interceptAuthCallback() {
  if (window.location.pathname === "/auth/callback") {
    const url = new URL(window.location.href);
    const code = url.searchParams.get("code");

    if (code) {
      handleAuthCallback(code);
    }
  }
}

async function handleAuthCallback(code: string) {
  setIsLoadingAuthInfo(true);
  const res = await fetch(
    `${import.meta.env.VITE_BACKEND_HOST}/auth/callback?code=${code}`,
  );
  const authInfo = await res.json();

  batch(() => {
    setAuthInfo(authInfo);
    setIsLoadingAuthInfo(false);
  });

  const redirect_url = window.localStorage.getItem(REDIRECT_KEY) ?? "/";
  console.log("REDIRECT:", redirect_url);

  const url = new URL(window.location.href);
  url.search = "";
  url.pathname = redirect_url;

  window.location.href = url.href;
}
