import { untrack } from "solid-js/web";
import { OpSeq } from "frontend-wasm";
import { EditorView } from "codemirror";
import { EditorSelection } from "@codemirror/state";
import { ViewUpdate } from "@codemirror/view";

import { authInfo } from "@features/auth/stores";
import { FileNode } from "@features/file-explorer/types";
import { onWsMessage, sendMessage } from "@features/ws/services";
import { wsSessionId } from "@features/ws/stores";
import { ClientMessageKind, ServerMessageKind } from "@features/ws/types";

import { Cursor } from "../types";
import {
  cursorsFiles,
  editingFiles,
  setCursorsFiles,
  setEditingFiles,
  setSyncFiles,
} from "../stores";
import { unicodeLength } from "./unicodeLength";
import { applyOperationToView, syncAnnotationType } from "./applyOperation";

let outstanding: OpSeq | null = null;
let accumulated_changes: OpSeq | null = null;

export function syncExtension(file: FileNode) {
  return EditorView.updateListener.of(anyEventHandler(file));
}

export function syncExtensionListener(view: EditorView, file: string) {
  onWsMessage(ServerMessageKind.Sync, (msg) => {
    if (msg.file === file) {
      const actual_revision = editingFiles[msg.file].synced_revision;

      if (msg.revision > actual_revision) {
        console.warn("History message has start greater than last operation.");
        return;
      }

      let othersCursors = { ...cursorsFiles[file] };

      let new_revision = actual_revision;
      for (
        let i = actual_revision - msg.revision;
        i < msg.actions.length;
        i++
      ) {
        let { user_id, operation } = msg.actions[i];
        new_revision++;
        if (user_id === untrack(wsSessionId)) {
          if (outstanding === null) {
            continue;
          }

          outstanding = accumulated_changes;
          accumulated_changes = null;

          if (outstanding) {
            sendMessage(ClientMessageKind.Sync, {
              file,
              revision: new_revision,
              actions: JSON.parse(outstanding.to_string()),
            });
          }
        } else {
          let opSeq = OpSeq.from_str(JSON.stringify(operation));

          if (outstanding) {
            const pair = outstanding.transform(opSeq)!;
            outstanding = pair.first();
            opSeq = pair.second();

            if (accumulated_changes) {
              const pair = accumulated_changes.transform(opSeq)!;
              accumulated_changes = pair.first();
              opSeq = pair.second();
            }
          }

          applyOperationToView(view, opSeq);

          const applyOp = (idx: number) =>
            Math.min(opSeq.transform_index(idx), opSeq.target_len());

          for (const [user, cursors] of Object.entries(othersCursors)) {
            const newCursors: Cursor[] = cursors.map((cursor) => ({
              from: applyOp(cursor.from),
              to: applyOp(cursor.to),
            }));

            othersCursors[user] = newCursors;
          }
        }
      }

      console.log(othersCursors);
      setCursorsFiles(file, othersCursors);

      setSyncFiles(file, view.state.doc.toString());
      setEditingFiles(msg.file, "synced_revision", new_revision);
    }
  });
}

function anyEventHandler(file: FileNode) {
  let lastCursors: EditorSelection;
  const handleCursor = (selection: EditorSelection) => {
    if (lastCursors && selection.eq(lastCursors)) return;
    lastCursors = selection;

    let cursors = selection.ranges.map(Cursor.from);

    setCursorsFiles(file.fullPath, untrack(authInfo)?.id, cursors);

    sendMessage(ClientMessageKind.SyncCursor, {
      file: file.fullPath,
      cursors: cursors.map(Cursor.into_rscursor),
    });
  };

  const handleOps = (update: ViewUpdate) => {
    let buffer = OpSeq.new();
    const oldContent = update.startState.doc.toString();
    const contentLength = unicodeLength(oldContent);

    buffer.retain(contentLength);

    let offset = 0;

    update.changes.iterChanges(
      (fromA, toA, _fromB, _toB, insert) => {
        const content = insert.sliceString(0, insert.length, "\n");

        fromA = buffer.transform_index(fromA);
        toA = buffer.transform_index(toA);

        const initial = unicodeLength(oldContent.slice(0, fromA));
        const deleted = unicodeLength(oldContent.slice(fromA, toA));
        const restLength = contentLength + offset - initial - deleted;

        const changeOp = OpSeq.new();
        changeOp.retain(initial);
        changeOp.delete(deleted);
        changeOp.insert(content);
        changeOp.retain(restLength);

        buffer = buffer.compose(changeOp)!;
        offset += changeOp.target_len() - changeOp.base_len();
      },
    );

    if (buffer.is_noop()) return;

    setSyncFiles(file.fullPath, update.state.doc.toString());

    // If there's is no pending to receive messages
    if (outstanding == null) {
      sendMessage(ClientMessageKind.Sync, {
        file: file.fullPath,
        revision: editingFiles[file.fullPath].synced_revision,
        actions: JSON.parse(buffer.to_string()),
      });
      outstanding = buffer;
    } else if (accumulated_changes == null) {
      accumulated_changes = buffer;
    } else {
      accumulated_changes = accumulated_changes.compose(buffer);
    }
  };

  return (update: ViewUpdate) => {
    if (update.transactions.some((v) => v.annotation(syncAnnotationType))) {
      return;
    }

    if (update.docChanged) {
      handleOps(update);
    }

    if (update.selectionSet || update.focusChanged) {
      handleCursor(update.state.selection);
    }
  };
}
