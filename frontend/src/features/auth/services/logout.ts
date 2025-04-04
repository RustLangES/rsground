import { batch } from "solid-js";
import { setAuthInfo, setIsLoadingAuthInfo } from "../stores";
import { generateRandomName } from "../utils/random_name";
import { loginGuest } from "./login_guest";

export async function logout() {
  setIsLoadingAuthInfo(true);

  const authInfo = await loginGuest(generateRandomName());

  batch(() => {
    setAuthInfo(authInfo);
    setIsLoadingAuthInfo(false);
  });
}
