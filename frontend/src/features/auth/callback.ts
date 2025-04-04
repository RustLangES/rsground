import { batch } from "solid-js";
import { setAuthInfo, setIsLoadingAuthInfo } from "./store";
import { authCallback } from "./service";

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

  const authInfo = await authCallback(code);

  batch(() => {
    setAuthInfo(authInfo);
    setIsLoadingAuthInfo(false);
  });

  const redirect_url = window.localStorage.getItem(REDIRECT_KEY) ?? "/";
  window.localStorage.removeItem(REDIRECT_KEY)

  const url = new URL(window.location.href);
  url.search = "";
  url.pathname = redirect_url;

  window.history.replaceState({}, "", url)
}

export function setRedirectUrl() {
  window.localStorage.setItem(REDIRECT_KEY, window.location.pathname)
}
