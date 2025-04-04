import { createStore } from "solid-js/store";

export interface File {
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

export interface Folder {
  /** Globally incremented id. */
  id: number;

  /**
   * UNIX-like path relative to home.
   * Includes the filename.
   */
  fullPath: string;

  /** The name of the file with extension. */
  name: string;

  children: FileNode[];

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

export type FileNode = {
  kind: FileNodeKind.File;
  data: File;
} | {
  kind: FileNodeKind.Folder;
  data: Folder;
};

export interface FileExplorerStore {
  nodes: FileNode[];
}

let nextId = 0;

export const fileExplorer = createStore<FileExplorerStore>({
  nodes: [
    {
      kind: FileNodeKind.Folder,
      data: {
        id: nextId++,
        fullPath: "FolderA",
        name: "FolderA",
        synced: true,
        children: [
          {
            kind: FileNodeKind.File,
            data: {
              id: nextId++,
              fullPath: "FolderA/FileA",
              filename: "FileA",
              synced: false,
              content: "",
            },
          },
          {
            kind: FileNodeKind.File,
            data: {
              id: nextId++,
              fullPath: "FolderA/FileB.rs",
              filename: "FileB.rs",
              synced: true,
              content: "",
            },
          },
        ],
      },
    },
    {
      kind: FileNodeKind.File,
      data: {
        id: nextId++,
        fullPath: "FileA",
        filename: "FileA",
        synced: true,
        content: "",
      },
    },
    {
      kind: FileNodeKind.File,
      data: {
        id: nextId++,
        fullPath: "FileB.rs",
        filename: "FileB.rs",
        synced: true,
        content: "",
      },
    },
  ],
}, { name: "FileExplorerStore" });

export function createNewFolder() {}
