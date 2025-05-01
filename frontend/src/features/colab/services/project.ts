import { untrack } from "solid-js/web";

import { authInfo } from "@features/auth/stores";
import { onWsMessage } from "@features/ws/services";
import { AccessLevel, ServerMessageKind } from "@features/ws/types";
import { BACKEND_HOST } from "@services";
import { showToast } from "@services/toast";

import { ProjectInfo } from "../types";
import { wsSessionId } from "@features/ws/stores";

onWsMessage(ServerMessageKind.UpdateAccess, (msg) => {
  if (msg.user_id === untrack(wsSessionId)) {
    if (msg.access === AccessLevel.Editor) {
      showToast("success", {
        titleText: "You have been granted to edit",
      });
    } else if (msg.access === AccessLevel.ReadOnly) {
      showToast("success", {
        titleText: "You have been granted to read",
      });
    }
  }
});

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


  const body = await res.text();

  if (res.status === 401) {
    try {
      return JSON.parse(body);
    } catch {}
  }

  if (!res.ok) {
    throw [res.status, body];
  }

  return JSON.parse(body);
}
