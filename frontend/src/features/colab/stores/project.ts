import { createSignal } from "solid-js";

import { AccessLevel } from "@features/ws/types";

import { ProjectInfo } from "../types";

export const [projectAccess, setProjectAccess] = createSignal<AccessLevel>(AccessLevel.Queue);

export const [projectId, setProjectId] = createSignal<string>(null);

export const [projectInfo, setProjectInfo] = createSignal<ProjectInfo>(null);
