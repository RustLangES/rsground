import { batch } from "solid-js";
import { setAuthInfo, setIsLoadingAuthInfo, goToRedirectUrl } from "../stores";
import { authCallback } from "../services";

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

  goToRedirectUrl()
}
