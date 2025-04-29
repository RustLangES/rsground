import { FileExplorerNode, FileNode, FileNodeKind, FolderNode } from "../types";

export function mergeFiles(
  nodes: string[],
  old: FileExplorerNode[],
  prefix = "",
): FileExplorerNode[] {
  const files: FileExplorerNode[] = [];
  const folders: FileExplorerNode[] = [];
  const pending_folders: Record<string, string[]> = {};

  // Collect files
  for (const node of nodes) {
    const [first, ...path] = node.split("/");

    if (path.length === 0) {
      files.push({
        kind: FileNodeKind.File as const,
        data: {
          fullPath: prefix + node,
          filename: first,
        },
      });
    } else {
      // Append remaining segments
      pending_folders[first] = [
        ...(pending_folders[first] ?? []),
        path.join("/"),
      ];
    }
  }

  // Collect folders
  for (const [name, children] of Object.entries(pending_folders)) {
    const oldFolder = (old.find((v) =>
      v.kind === FileNodeKind.Folder && v.data.name === name
    )?.data as FolderNode)?.children ?? [];

    folders.push({
      kind: FileNodeKind.Folder as const,
      data: {
        name,
        fullPath: prefix + name,
        children: mergeFiles(children, oldFolder, prefix + name + "/"),
      },
    });
  }

  // Keep empty folders from the same session
  for (const oldFolder of old) {
    if (
      oldFolder.kind === FileNodeKind.Folder &&
      oldFolder.data.children.length === 0 &&
      !pending_folders[oldFolder.data.name]
    ) {
      folders.push(oldFolder);
    }
  }

  files.sort(({ data: a }, { data: b }) =>
    (a as FileNode).filename.localeCompare((b as FileNode).filename)
  );

  folders.sort(({ data: a }, { data: b }) =>
    (a as FolderNode).name.localeCompare((b as FolderNode).name)
  );

  return [...folders, ...files];
}
