import { observable, untrack } from "solid-js";

import { authInfo } from "@features/auth/stores";
import { AuthInfo } from "@features/auth/types";

import { createProject } from "../services";
import { setProjectId } from "../stores";

export function interpectProjectRoutes() {
  if (window.location.pathname === "/") {
    createProjectWith(untrack(authInfo));
    observable(authInfo).subscribe(createProjectWith);
    return;
  }

  let segments = window.location.pathname.split("/");
  segments.shift();

  let projectId = segments.shift();
  let maybeAction = segments.shift();

  if (maybeAction === "fork") {
    // TODO: fork project
  }

  setProjectId(projectId);
}

async function createProjectWith(authInfo: AuthInfo) {
  if (!!authInfo?.jwt) {
    let projectId = await createProject(authInfo.jwt);

    window.location.pathname = "/" + projectId;
  }
}
