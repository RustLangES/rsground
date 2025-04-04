import { BACKEND_HOST } from "@services";
import { saveRedirectUrl } from "../stores";

export async function loginGithub() {
  saveRedirectUrl();

  window.location.href = `${BACKEND_HOST}/auth`
}

