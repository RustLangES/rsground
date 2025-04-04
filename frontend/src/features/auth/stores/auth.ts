import { createSignal, observable } from "solid-js";
import { AuthInfo } from "../types";

export const AUTH_KEY = "auth";

// Use previous auth info in local storage
let prevAuthInfo: AuthInfo | null = null;
try {
  const stored = window.localStorage.getItem(AUTH_KEY);

  if (stored) {
    prevAuthInfo = JSON.parse(stored);
  }
} catch (e) {
  // Just log error for debug, but continue
  console.error(e);
}

export const [authInfo, setAuthInfo] = createSignal<AuthInfo | null>(
  prevAuthInfo,
);

export const isGithubLogged = () => !!authInfo()?.avatar_url;

export const [isLoadingAuthInfo, setIsLoadingAuthInfo] = createSignal(false);

// Sync auth info in local storage
observable(authInfo).subscribe((authInfo) => {
  if (authInfo) {
    window.localStorage.setItem(AUTH_KEY, JSON.stringify(authInfo));
  } else {
    window.localStorage.removeItem(AUTH_KEY);
  }
});
