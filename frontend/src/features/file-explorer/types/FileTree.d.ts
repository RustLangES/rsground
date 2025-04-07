export interface FileNode {
  /** Globally incremented id. */
  id: number;

  /**
   * UNIX-like path relative to home.
   * Includes the filename.
   */
  fullPath: string;

  /** The name of the file with extension. */
  filename: string;

  /** Last synced content */
  content: string;

  /**
   * Whether or not is already synced with backend.
   * It is set once the backend sends successful code.
   */
  synced: boolean;
}

export interface FolderNode {
  /** Globally incremented id. */
  id: number;

  /**
   * UNIX-like path relative to home.
   * Includes the filename.
   */
  fullPath: string;

  /** The name of the file with extension. */
  name: string;

  children: FileExplorerNode[];

  /**
   * Whether or not is already synced with backend.
   * It is set once the backend sends successful code.
   */
  synced: boolean;
}

export enum FileNodeKind {
  File,
  Folder,
}

export type FileExplorerNode = {
  kind: FileNodeKind.File;
  data: FileNode;
} | {
  kind: FileNodeKind.Folder;
  data: FolderNode;
};
