import { RsCursor, UserOperation } from "@features/editor/types";
import { AccessLevel } from "./access";

export enum ServerMessageKind {
  Error = "error",
  ProjectConfig = "project_config",
  ProjectFiles = "project_files",
  UpdateAccess = "update_access",
  UserConnected = "user_connected",
  RequestAccess = "request_access",
  Sync = "sync",
  SyncCursors = "sync_cursors",
  Welcome = "welcome",
}

export type ServerMessage<S extends ServerMessageKind = ServerMessageKind> = {
  [ServerMessageKind.Error]: {
    action: ServerMessageKind.Error;
    message: string;
  };
  [ServerMessageKind.ProjectConfig]: {
    action: ServerMessageKind.ProjectConfig;
    name: string;
    is_public: boolean;
    password?: string;
  };
  [ServerMessageKind.ProjectFiles]: {
    action: ServerMessageKind.ProjectFiles;
    files: Record<string, DocumentInfo>;
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
  [ServerMessageKind.RequestAccess]: {
    action: ServerMessageKind.RequestAccess;
    user_id: string;
    user_name: string;
  };
  [ServerMessageKind.Sync]: {
    action: ServerMessageKind.Sync;
    file: string;
    revision: number;
    actions: Array<UserOperation>;
  };
  [ServerMessageKind.SyncCursors]: {
    action: ServerMessageKind.SyncCursors;
    file: string;
    cursors: Record<string, Array<RsCursor>>;
  };
  [ServerMessageKind.Welcome]: {
    action: ServerMessageKind.Welcome;
    session_id: string;
    files: Record<string, DocumentInfo>;
    users: Record<string, AccessLevel>;
  };
}[S];

export interface DocumentInfo {
  text: string,
  revision: number,
}
