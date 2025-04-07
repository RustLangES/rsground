import { createStore } from "solid-js/store";
import { FileExplorerStore, FileNodeKind } from "../types";

let nextId = 0;

export const fileExplorer = createStore<FileExplorerStore>({
  nodes: [
    {
      kind: FileNodeKind.Folder as const,
      data: {
        id: nextId++,
        fullPath: "FolderA",
        name: "FolderA",
        synced: true,
        children: [
          {
            kind: FileNodeKind.File as const,
            data: {
              id: nextId++,
              fullPath: "FolderA/FileA",
              filename: "FileA",
              synced: false,
              content: "",
            },
          },
          {
            kind: FileNodeKind.File as const,
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
      kind: FileNodeKind.File as const,
      data: {
        id: nextId++,
        fullPath: "FileA",
        filename: "FileA",
        synced: true,
        content: "",
      },
    },
    {
      kind: FileNodeKind.File as const,
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
