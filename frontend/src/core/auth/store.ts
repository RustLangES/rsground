import { createSignal, observable, useTransition } from "solid-js";

export const AUTH_KEY = "auth";

export interface AuthInfo {
  jwt?: string;
  username?: string;
  name?: string;
  avatar_url: string;
}

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

export const [isLoadingAuthInfo, setIsLoadingAuthInfo] = createSignal(false);

// Sync auth info in local storage
observable(authInfo).subscribe((authInfo) => {
  window.localStorage.setItem(AUTH_KEY, JSON.stringify(authInfo));
})
