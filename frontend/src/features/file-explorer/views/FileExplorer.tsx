import { Index } from "solid-js";

import { ContextMenu } from "@features/context-menu/views";
import { openFile } from "@features/editor/services";
import { FolderPlusIcon } from "@icons/FolderPlus";
import { FolderMinusIcon } from "@icons/FolderMinus";
import { BrandsRustIcon } from "@icons/BrandsRust";
import { FileLinesIcon } from "@icons/FileLines";

import { fileExplorer} from "../stores";
import { FileExplorerNode, FileNode, FileNodeKind, FolderNode } from "../types";

import styles from "./FileExplorer.module.sass";

function RenderFolder({ data }: { data: FolderNode }) {
  return (
    <li class={styles.entry_folder}>
      <details>
        <ContextMenu
          as="summary"
          options={{
            [data.name]: { disabled: true },
            "Add File": {},
            "Copy": {},
            "Paste": {},
            "Rename": { level: "warning" },
            "Delete": { level: "error" },
          }}
        >
          <FolderPlusIcon class={styles.closed_folder} />
          <FolderMinusIcon class={styles.opened_folder} />

          <span>{data.name}</span>
        </ContextMenu>
        <ul>
          <RenderNodes nodes={data.children} />
        </ul>
      </details>
    </li>
  );
}

function RenderFile({ data }: { data: FileNode }) {
  return (
    <ContextMenu
      as="li"
      classList={{
        [styles.entry]: true,
        [styles.entry_syncing]: !data.synced,
      }}
      options={{
        [data.filename]: { disabled: true },
        "Copy": {},
        "Paste": {},
        "Rename": { level: "warning" },
        "Delete": { level: "error" },
      }}
      onClick={() => openFile(data.fullPath)}
    >
      {data.filename.endsWith(".rs") ? <BrandsRustIcon /> : <FileLinesIcon />}
      <span>{data.filename}</span>
    </ContextMenu>
  );
}

function RenderNodes({ nodes }: { nodes: FileExplorerNode[] }) {
  return (
    <Index each={nodes}>
      {(node_) => {
        const node = node_();
        return node.kind == FileNodeKind.Folder
          ? <RenderFolder data={node.data} />
          : <RenderFile data={node.data} />;
      }}
    </Index>
  );
}

export function FileExplorer() {
  return (
    <ContextMenu
      as="ul"
      class={styles.container}
      options={{
        "Add File": {},
        "Add Folder": {},
      }}
    >
      <RenderNodes nodes={fileExplorer.nodes} />
    </ContextMenu>
  );
}
