export interface AuthInfo {
  jwt: string;
  is_guest: boolean;
  name: string;

  avatar_url?: string;
}
