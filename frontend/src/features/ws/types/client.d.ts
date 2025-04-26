import { OtOperation } from "@features/editor/types";
import { AccessLevel } from "./access";

export enum ClientMessageKind {
  PermitAccess = "permit_access",
  FileCreate = "file_create",
  FileDelete = "file_delete",
  Sync = "sync",
  SyncFiles = "sync_files",
}

export type ClientMessage<S extends ClientMessageKind = ClientMessageKind> = {
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
    actions: Array<OtOperation>;
  };
  [ClientMessageKind.SyncFiles]: {
    action: ClientMessageKind.SyncFiles;
  };
}[S];
