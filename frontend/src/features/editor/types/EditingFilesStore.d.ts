import { Action } from "@features/ws/types";

export type EditingFilesStore = {
  /** Pending actions for `k` */
  [k: string]: Action[];
};
