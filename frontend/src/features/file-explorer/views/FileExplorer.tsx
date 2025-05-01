import { For } from "solid-js";

import { projectAccess } from "@features/colab/stores";
import { ContextMenu } from "@features/context-menu/views";
import { openFile } from "@features/editor/services";
import { onWsMessage } from "@features/ws/services";
import { AccessLevel, ServerMessageKind } from "@features/ws/types";
import { FolderPlusIcon } from "@icons/FolderPlus";
import { FolderMinusIcon } from "@icons/FolderMinus";
import { BrandsRustIcon } from "@icons/BrandsRust";
import { FileLinesIcon } from "@icons/FileLines";

import { createNewFile, syncFiles } from "../services";
import { fileExplorer } from "../stores";
import { FileExplorerNode, FileNode, FileNodeKind, FolderNode } from "../types";

import styles from "./FileExplorer.module.sass";

function RenderFolder(props: { data: FolderNode }) {
  return (
    <li class={styles.entry_folder}>
      <details>
        <ContextMenu
          as="summary"
          useRightClick={projectAccess() === AccessLevel.Editor}
          options={{
            [props.data.name]: { disabled: true },
            "Add File": {},
            "Copy": {},
            "Paste": {},
            "Rename": { level: "warning" },
            "Delete": { level: "error" },
          }}
        >
          <FolderPlusIcon class={styles.closed_folder} />
          <FolderMinusIcon class={styles.opened_folder} />

          <span>{props.data.name}</span>
        </ContextMenu>
        <ul>
          <RenderNodes nodes={props.data.children} />
        </ul>
      </details>
    </li>
  );
}

function RenderFile(props: { data: FileNode }) {
  return (
    <ContextMenu
      as="li"
      useRightClick={projectAccess() === AccessLevel.Editor}
      class={styles.entry}
      options={{
        [props.data.filename]: { disabled: true },
        "Copy": {},
        "Paste": {},
        "Rename": { level: "warning" },
        "Delete": { level: "error" },
      }}
      onClick={() => openFile(props.data.fullPath)}
    >
      {props.data.filename.endsWith(".rs")
        ? <BrandsRustIcon />
        : <FileLinesIcon />}
      <span>{props.data.filename}</span>
    </ContextMenu>
  );
}

function RenderNodes(props: { nodes: FileExplorerNode[] }) {
  return (
    <For each={props.nodes}>
      {(node) => {
        return node.kind == FileNodeKind.Folder
          ? <RenderFolder data={node.data} />
          : <RenderFile data={node.data} />;
      }}
    </For>
  );
}

export function FileExplorer() {
  onWsMessage(ServerMessageKind.ProjectFiles, (msg) => syncFiles(msg.files));

  return (
    <ContextMenu
      as="ul"
      useRightClick={projectAccess() === AccessLevel.Editor}
      class={styles.container}
      options={{
        "Add File": {
          onClick() {
            const file = prompt("File path:");
            if (file) {
              createNewFile(file);
            }
          },
        },
        "Add Folder": {},
      }}
    >
      <RenderNodes nodes={fileExplorer.nodes} />
    </ContextMenu>
  );
}
