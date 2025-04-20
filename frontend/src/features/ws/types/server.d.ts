import { AccessLevel } from "./access";
import { Action } from "./action";

export enum ServerMessageKind {
  Error = "error",
  JoinedProject = "joined_project",
  ProjectCreated = "project_created",
  ProjectFiles = "project_files",
  ProjectForked = "project_forked",
  Update = "update",
  UpdateAccess = "update_access",
  UserConnected = "user_connected",
  SyncActions = "sync_actions",
}

export type ServerMessage<S extends ServerMessageKind = ServerMessageKind> = {
  [ServerMessageKind.Error]: {
    action: ServerMessageKind.Error;
    message: string;
  };
  [ServerMessageKind.JoinedProject]: {
    action: ServerMessageKind.JoinedProject;
    access: AccessLevel;
    user_id: string;
  };
  [ServerMessageKind.ProjectCreated]: {
    action: ServerMessageKind.ProjectCreated;
    project_id: string;
  };
  [ServerMessageKind.ProjectFiles]: {
    action: ServerMessageKind.ProjectFiles;
    files: Array<string>;
  };
  [ServerMessageKind.ProjectForked]: {
    action: ServerMessageKind.ProjectForked;
    project_id: string;
  };
  [ServerMessageKind.Update]: {
    action: ServerMessageKind.Update;
    file: string;
    content: string;
  };
  [ServerMessageKind.UpdateAccess]: {
    action: ServerMessageKind.UpdateAccess;
    access: AccessLevel;
    user_id: string;
  };
  [ServerMessageKind.UserConnected]: {
    action: ServerMessageKind.UserConnected;
    user_id: string;
  };
  [ServerMessageKind.SyncActions]: {
    action: ServerMessageKind.SyncActions;
    file: string;
    actions: Array<Action>;
  };
}[S];
