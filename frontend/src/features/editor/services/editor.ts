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
import { applyOperationToString } from "../utils";
import { OpSeq } from "frontend-wasm";

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

    setEditingFiles(msg.file, "synced_revision", msg.revision);

    let content = unwrap(syncFiles)[msg.file];

    if (content != null) {
      content = msg.actions.reduce(
        (content, action) =>
          applyOperationToString(
            content,
            OpSeq.from_str(JSON.stringify(action.operation)),
          ),
        content,
      );
    }

    setSyncFiles(msg.file, content ?? "");
  });
}
