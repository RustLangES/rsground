import { BACKEND_HOST } from "@services";
import { AuthInfo } from "../types";

export async function loginGuest(guest_name: string): Promise<AuthInfo> {
  const res = await fetch(
    `${BACKEND_HOST}/login-guest`,
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

  return await res.json();
}
