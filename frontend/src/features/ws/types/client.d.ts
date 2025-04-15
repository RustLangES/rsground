import { AccessLevel } from "./access";

export type ClientMessage =
  | { action: "create_project"; name: string }
  | { action: "delete"; file: string; range_start: number; range_end: number }
  | { action: "fork_project"; project_id: string }
  | { action: "get_project_files" }
  | { action: "insert"; file: string; pos: number; text: string }
  | { action: "join_project"; project_id: string; password?: string }
  | { action: "permit_access"; user_id: string; access: AccessLevel }
  | { action: "sync"; file: String; last_timestamp: number };
