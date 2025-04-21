import { BACKEND_HOST } from "@services";
import { AuthInfo } from "../types";

export async function fetchMe(jwt: string): Promise<AuthInfo> {
  const res = await fetch(
    `${BACKEND_HOST}/auth/me`,
    {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${jwt}`,
      },
    },
  );

  if (res.ok) {
    return await res.json();
  } else {
    return null;
  }
}
