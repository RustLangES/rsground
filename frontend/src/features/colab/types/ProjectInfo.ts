import { AccessLevel } from "@features/ws/types";

export interface ProjectInfo {
  id: string,
  name: string,
  owner: string,
  allowed_users: Record<string, AccessLevel>,
  is_public: boolean,
  password?: string,
}
