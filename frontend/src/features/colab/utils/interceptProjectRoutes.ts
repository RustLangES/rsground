import { observable } from "solid-js";

import { authInfo } from "@features/auth/stores";
import { AuthInfo } from "@features/auth/types";

import { createProject, fetchProject } from "../services";
import { setProjectId } from "../stores";

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
  }

  fetchProject(projectId).then((project) => {
    // Check if has access to project
    if (project.allowed_users == null) {
      // TODO: Pending permission
      alert("TODO: Not allowed")
    }

    setProjectId(projectId);
  }).catch((err) => {
    if (err === 404) {
      alert("Project not found. Creating new one");
      observable(authInfo).subscribe(createProjectWith);
    } else if (err == 401) {
      // TODO: Invalid password
      console.error("Invalid password");
    } else {
      console.error(err);
    }
  });
}

async function createProjectWith(authInfo: AuthInfo) {
  if (!!authInfo?.jwt) {
    let projectId = await createProject(authInfo.jwt);

    window.location.pathname = "/" + projectId;
  }
}
