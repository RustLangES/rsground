export interface FileNode {
  /**
   * UNIX-like path relative to home.
   * Includes the filename.
   */
  fullPath: string;

  /** The name of the file with extension. */
  filename: string;
}

export interface FolderNode {
  /**
   * UNIX-like path relative to home.
   * Includes the filename.
   */
  fullPath: string;

  /** The name of the file with extension. */
  name: string;

  children: FileExplorerNode[];
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
