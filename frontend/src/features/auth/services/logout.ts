import { setIsLoadingAuthInfo } from "../stores";
import { generateRandomName } from "../utils/random_name";
import { loginGuest } from "./login_guest";

export async function logout() {
  setIsLoadingAuthInfo(true);

  await loginGuest(generateRandomName());

  setIsLoadingAuthInfo(false);
}
