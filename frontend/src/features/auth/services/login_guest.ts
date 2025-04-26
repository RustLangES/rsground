import { BACKEND_HOST } from "@services";
import { AuthInfo } from "../types";
import { setAuthInfo } from "../stores";

export async function loginGuest(guest_name: string): Promise<AuthInfo> {
  const res = await fetch(
    `${BACKEND_HOST}/auth/guest`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        guest_name,
      }),
    },
  );

  const authInfo = await res.json();

  setAuthInfo(authInfo);

  return authInfo;
}
