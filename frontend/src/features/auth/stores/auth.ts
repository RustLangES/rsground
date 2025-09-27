import { createSignal } from "solid-js";
import { AuthInfo } from "../types";
import { createLocalStoredSignal, createSessionStoredSignal } from "@utils/createLocalStoredSignal";
import { randomAvatar } from "../utils";

export const AUTH_KEY = "auth";
export const AVATAR_KEY = "auth.avatar";

export const [authInfo, setAuthInfo] = createLocalStoredSignal<AuthInfo | null>(
  AUTH_KEY,
  null,
  (v) => JSON.parse(v),
  (v) => JSON.stringify(v)
);

export const [guestAvatar, _] = createSessionStoredSignal<string>(
  AVATAR_KEY,
  randomAvatar(),
  (v) => v,
  (v) => v
);

export const isGithubLogged = () => !!authInfo()?.avatar_url;

export const [isLoadingAuthInfo, setIsLoadingAuthInfo] = createSignal(false);
