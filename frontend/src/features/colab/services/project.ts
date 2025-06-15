import { batch, untrack } from "solid-js";
import SWAL from "sweetalert2";

import { authInfo } from "@features/auth/stores";
import { onWsMessage, startWebsocket } from "@features/ws/services";
import { AccessLevel, ServerMessageKind } from "@features/ws/types";
import { BACKEND_HOST } from "@services";
import { showModal } from "@services/modal";
import { showToast } from "@services/toast";

import { ProjectInfo } from "../types";
import {
  isProjectOwner,
  projectAccess,
  projectInfo,
  setIsProjectOwner,
  setProjectAccess,
  setProjectInfo,
} from "../stores";
import { WaitingAccess } from "../views";

onWsMessage(ServerMessageKind.UpdateAccess, (msg) => {
  if (!isProjectOwner()) {
    const oldAccess = untrack(projectAccess);

    setProjectAccess(msg.access);

    if (oldAccess === AccessLevel.Queue && msg.access !== AccessLevel.Queue) {
      window.location.reload();
      return;
    }

    if (msg.access === AccessLevel.Editor) {
      showToast("success", {
        titleText: "You have been granted to edit",
      });
    } else if (msg.access === AccessLevel.ReadOnly) {
      showToast("success", {
        titleText: "You have been granted to read",
      });
    } else if (msg.access === AccessLevel.Queue) {
      showToast("error", {
        titleText: "You have been kicked",
      });
    }
    return;
  }

  setProjectInfo("users", msg.user_id, 1, msg.access);

  projectInfo.requests[msg.user_id] &&
    setProjectInfo("requests", msg.user_id, undefined);
});

onWsMessage(ServerMessageKind.ProjectConfig, (msg) => {
  setProjectInfo({
    name: msg.name,
    is_public: msg.is_public,
    password: msg.password,
  });
});

onWsMessage(ServerMessageKind.RequestAccess, (msg) => {
  showToast("info", {
    titleText: `${msg.user_name} is requesting access`,
    timer: 5_000,
  }).then(() => {
    setProjectInfo("requests", msg.user_id, msg.user_name);
  });
});

export function setProject(project: ProjectInfo, shouldFork: boolean) {
  // Check if has access to project
  if (project.users == null) {
    // TODO: Pending permission, listen to permission granted.
    // Once user is allowed, should restart websocket connection
    // for receive welcome
    setProjectInfo("id", project.id);
    showModal(WaitingAccess, {
      allowOutsideClick: false,
    });
    return;
  }

  if (shouldFork) {
    forkProject(project.id);
    return;
  }

  batch(() => {
    if (project.owner === untrack(authInfo).id) {
      setIsProjectOwner(true);
    }

    setProjectAccess(
      project.users[untrack(authInfo).id]?.[1] ?? AccessLevel.Queue,
    );
    setProjectInfo(project);
  });

  // Close current modal, maybe it is password
  // or waiting screen
  SWAL.close();

  startWebsocket();
}

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
  let res = await fetch(`${BACKEND_HOST}/project/${project_id}?p=${password}`, {
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

export async function forkProject(project_id: string) {
  let res = await fetch(`${BACKEND_HOST}/fork/${project_id}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${untrack(authInfo)?.jwt}`,
    },
  });

  if (!res.ok) {
    return;
  }

  const { id } = await res.json();

  location.pathname = "/" + id;
}
