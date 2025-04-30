export type EditingFilesStore = {
  /** Action history for `k` */
  [k: string]: {
    /** Last synced revision */
    synced_revision: number;
    /** Is any code editor attached to this file */
    editor_open: boolean;
  };
};
