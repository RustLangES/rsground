import { AccessLevel } from "./access";
import { Action } from "./action";

export type ServerMessage =
  | { action: "error"; message: string }
  | { action: "joined_project"; access: AccessLevel; user_id: string }
  | { action: "project_created"; project_id: string }
  | { action: "project_files"; files: Array<string> }
  | { action: "project_forked"; project_id: string }
  | { action: "update"; file: string; content: string }
  | { action: "update_access"; access: AccessLevel; user_id: string }
  | { action: "user_connected"; user_id: string }
  | { action: "sync_actions"; file: string; actions: Array<Action> };
