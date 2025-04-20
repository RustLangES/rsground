import { createStore, SetStoreFunction, Store } from "solid-js/store";
import { FileExplorerNode, FileExplorerStore, FileNodeKind } from "../types";

let nextId = 0;

export const [fileExplorer, setFileExplorer] = createStore<FileExplorerStore>({
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
            },
          },
          {
            kind: FileNodeKind.File as const,
            data: {
              id: nextId++,
              fullPath: "FolderA/FileB.rs",
              filename: "FileB.rs",
              synced: true,
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
      },
    },
    {
      kind: FileNodeKind.File as const,
      data: {
        id: nextId++,
        fullPath: "main.rs",
        filename: "main.rs",
        synced: true,
      },
    },
  ],
}, { name: "FileExplorerStore" });

/**
 * Get synced store to `fullpath`.
 *
 * @param fullpath Relative path from home, without `/` at start
 */
export function getNodeByPath(
  fullpath: string,
): [Store<FileExplorerNode>, SetStoreFunction<FileExplorerNode>] {
  const segments = fullpath.split("/");

  // Trim target filename, leave just parent folders
  segments.pop();

  let children = fileExplorer.nodes;

  // Get all folders before target
  s: for (const segment of segments) {
    // Search through last children...
    for (const child of children) {
      // For the next folder with the segment name
      if (child.kind === FileNodeKind.Folder && child.data.name == segment) {
        children = child.data.children;

        // skip to next segment
        continue s;
      }
    }

    // If there're not target child, then fail here
    return null;
  }

  // Search through last children...
  for (const child of children) {
    // For the exact target
    if (child.data.fullPath == fullpath) {
      return createStore(child, { name: fullpath });
    }
  }

  return null;
}

export function createNewFolder() {}
