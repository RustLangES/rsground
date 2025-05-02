import { OtOperation, RsCursor } from "@features/editor/types";
import { AccessLevel } from "./access";

export enum ServerMessageKind {
  Error = "error",
  ProjectFiles = "project_files",
  UpdateAccess = "update_access",
  UserConnected = "user_connected",
  Sync = "sync",
  SyncCursors = "sync_cursors",
  Welcome = "welcome",
}

export type ServerMessage<S extends ServerMessageKind = ServerMessageKind> = {
  [ServerMessageKind.Error]: {
    action: ServerMessageKind.Error;
    message: string;
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
  [ServerMessageKind.Sync]: {
    action: ServerMessageKind.Sync;
    file: string;
    revision: number;
    actions: Array<OtOperation>;
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
