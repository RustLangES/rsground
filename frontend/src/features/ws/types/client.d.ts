import { RsCursor, UserOperation } from "@features/editor/types";
import { AccessLevel } from "./access";

export enum ClientMessageKind {
  Config = "config",
  PermitAccess = "permit_access",
  FileCreate = "file_create",
  FileDelete = "file_delete",
  Sync = "sync",
  SyncCursor = "sync_cursor",
  SyncFiles = "sync_files",
}

export type ClientMessage<S extends ClientMessageKind = ClientMessageKind> = {
  [ClientMessageKind.Config]: {
    action: ClientMessageKind.Config;
    name?: string;
    is_public?: boolean;
    password?: string;
  };
  [ClientMessageKind.PermitAccess]: {
    action: ClientMessageKind.PermitAccess;
    user_id: string;
    access: AccessLevel;
  };
  [ClientMessageKind.FileCreate]: {
    action: ClientMessageKind.FileCreate;
    file: string;
  };
  [ClientMessageKind.FileDelete]: {
    action: ClientMessageKind.FileDelete;
    file: string;
  };
  [ClientMessageKind.Sync]: {
    action: ClientMessageKind.Sync;
    file: string;
    revision: number;
    actions: Array<UserOperation>;
  };
  [ClientMessageKind.SyncCursor]: {
    action: ClientMessageKind.SyncCursor;
    file: string;
    cursors: Array<RsCursor>;
  };
  [ClientMessageKind.SyncFiles]: {
    action: ClientMessageKind.SyncFiles;
  };
}[S];
