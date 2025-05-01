import { observable, untrack } from "solid-js";

import { authInfo } from "@features/auth/stores";
import { AuthInfo } from "@features/auth/types";
import { AccessLevel } from "@features/ws/types";
import { showToast } from "@services/toast";

import { createProject, fetchProject } from "../services";
import { setProjectAccess, setProjectId, setProjectInfo } from "../stores";

export function interpectProjectRoutes() {
  if (window.location.pathname === "/") {
    observable(authInfo).subscribe(createProjectWith);
    return;
  }

  let segments = window.location.pathname.split("/");
  segments.shift();

  let projectId = segments.shift();
  let maybeAction = segments.shift();

  if (maybeAction === "fork") {
    // TODO: fork project
    showToast("debug", {
      titleText: "Fork project",
      text: "Not implemented yet",
    });
    return;
  }

  fetchProject(projectId).then((project) => {
    // Check if has access to project
    if (project.allowed_users == null) {
      // TODO: Pending permission, listen to permission granted.
      // Once user is allowed, should restart websocket connection
      // for receive welcome
      setProjectId(projectId);
      showToast("error", {
        titleText: "Not access to project",
      });
      return;
    }

    setProjectAccess(
      project.allowed_users[untrack(authInfo).id] ?? AccessLevel.Queue,
    );
    setProjectId(projectId);
    setProjectInfo(project);
  }).catch((err: [number, string]) => {
    if (err instanceof Array) {
      if (err[0] === 404) {
        showToast("error", {
          titleText: "Project not found. Creating new one",
          timer: 2_000,
        }).then(() => {
          createProjectWith(untrack(authInfo));
        });
        return;
      }

      // Just retry until auto-logged
      if (err[0] === 401 && err[1] == "Invalid token") {
        interpectProjectRoutes();
        return;
      }

      if (err[0] == 401) {
        showToast("error", {
          titleText: "Invalid password",
        });
        return;
      }
    }

    console.error(err);
    showToast("error", {
      titleText: "Unexpected error",
      text: "Contact to developers",
    });
  });
}

async function createProjectWith(authInfo: AuthInfo) {
  if (!!authInfo?.jwt) {
    let projectId = await createProject(authInfo.jwt);

    window.location.pathname = "/" + projectId;
  }
}
