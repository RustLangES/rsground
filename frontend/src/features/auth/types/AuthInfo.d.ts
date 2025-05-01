export interface AuthInfo {
  jwt: string;
  id: string;
  is_guest: boolean;
  name: string;

  avatar_url?: string;
}
