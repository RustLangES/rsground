import { untrack } from "solid-js/web";

import { authInfo } from "@features/auth/stores";
import { BACKEND_HOST } from "@services";

import { ProjectInfo } from "../types";

export async function createProject(
  owner: string,
  name: string = "Unnamed",
): Promise<string> {
  let res = await fetch(`${BACKEND_HOST}/create/${name}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${owner}`,
    },
  });

  if (!res.ok) {
    return null;
  }

  return (await res.json()).id;
}

export async function fetchProject(
  project_id: string,
  password = "",
): Promise<ProjectInfo> {
  let res = await fetch(`${BACKEND_HOST}/project/${project_id}?${password}`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${untrack(authInfo)?.jwt}`,
    },
  });

  if (res.status === 401) {
    try {
      return await res.json();
    } catch {}
  }

  if (!res.ok) {
    throw res.status;
  }

  return await res.json();
}
