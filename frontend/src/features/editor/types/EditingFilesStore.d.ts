export type EditingFilesStore = {
  /** Action history for `k` */
  [k: string]: {
    /** Last synced revision */
    synced_revision: number;
    /** Local revisions ahead of synced */
    local_revision: number;
  };
};
