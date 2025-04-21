import { fetchMe, loginGithub, loginGuest } from "../services";
import { authInfo } from "../stores";
import { generateRandomName } from "./random_name";

export async function checkForAuth() {
  let { jwt, name, is_guest } = authInfo() ?? {};

  if (!jwt) {
    await loginGuest(generateRandomName());
    return;
  }

  let user_info = await fetchMe(jwt);

  if (user_info != null) return;

  if (is_guest) {
    await loginGuest(name);
  } else {
    await loginGithub()
  }
}
