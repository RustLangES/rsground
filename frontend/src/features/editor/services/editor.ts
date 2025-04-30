import { untrack } from "solid-js";
import { unwrap } from "solid-js/store";

import { dockview } from "@features/panels/stores";
import { onWsMessage } from "@features/ws/services";
import { ServerMessageKind } from "@features/ws/types";

import {
  editingFiles,
  setEditingFiles,
  setSyncFiles,
  syncFiles,
} from "../stores";
import { OtOperationKind } from "../types";

export async function openFile(filepath: string) {
  const id = `file:${filepath}`;
  const filename = filepath.split("/").pop();

  if (!untrack(dockview).getPanel(id)) {
    untrack(dockview).addPanel({
      id,
      component: "code",
      title: filename,
      position: { direction: "above", referencePanel: "output" },
    });
  }
}

export function startReceivingSync() {
  onWsMessage(ServerMessageKind.Sync, (msg) => {
    if (unwrap(editingFiles)[msg.file].editor_open) return;

    const local_revision = editingFiles[msg.file].synced_revision;
    const desyncronized_history = msg.actions.slice(local_revision);

    setEditingFiles(msg.file, "synced_revision", msg.revision);

    let content = unwrap(syncFiles)[msg.file] ?? "";

    for (const action of desyncronized_history) {
      if (action.kind === OtOperationKind.Insert) {
        content = content.slice(0, action.from) + action.text + content.slice(action.from);
      } else {
        content = content.slice(0, action.from) + content.slice(action.to);
      }
    }

    setSyncFiles(msg.file, content);
  });
}
