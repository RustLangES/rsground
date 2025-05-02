import { untrack } from "solid-js/web";
import { EditorView } from "codemirror";
import { Annotation, EditorSelection } from "@codemirror/state";
import { ViewUpdate } from "@codemirror/view";

import { FileNode } from "@features/file-explorer/types";
import { onWsMessage, sendMessage } from "@features/ws/services";
import { wsSessionId } from "@features/ws/stores";
import {
  ClientMessageKind,
  ServerMessage,
  ServerMessageKind,
} from "@features/ws/types";

import { Cursor, OtOperation, OtOperationKind } from "../types";
import { editingFiles, setCursorsFiles, setEditingFiles, setSyncFiles } from "../stores";
import { optimizeOps } from "./optimizeOps";
import { transformIndex } from "./transformIndex";
import { authInfo } from "@features/auth/stores";

const ownerAnnotation = Annotation.define<string>();

export function syncExtension(file: FileNode) {
  return EditorView.updateListener.of(anyEventHandler(file));
}

export function syncExtensionListener(view: EditorView, file: string) {
  onWsMessage(ServerMessageKind.Sync, (msg) => {
    if (msg.file === file) {
      receiveOps(view, file, msg);
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
      cursors: cursors.map(Cursor.into_rscursor)
    })

    console.log("CURSORS", cursors);
  };

  let accumulated_changes: OtOperation[] = [];

  const handleOps = () => {
    if (accumulated_changes.length === 0) return;

    // Prepare ops in the most compact form
    const ops = sendableOps(accumulated_changes);
    accumulated_changes = [];

    if (ops.length === 0) return;

    sendMessage(ClientMessageKind.Sync, {
      file: file.fullPath,
      revision: editingFiles[file.fullPath].synced_revision,
      actions: ops,
    });
  };

  const realEventHandler = (update: ViewUpdate) => {
    setSyncFiles(file.fullPath, update.state.doc.toString());

    handleOps();
    if (update.selectionSet || update.focusChanged) {
      handleCursor(update.state.selection);
    }
  };

  let cb: NodeJS.Timeout;
  return (update: ViewUpdate) => {
    if (update.transactions.some((v) => !!v.annotation(ownerAnnotation))) {
      return;
    }
    if (cb) clearTimeout(cb);

    update.changes.iterChanges(
      (fromA, toA, _fromB, _toB, insert) => {
        const content = insert.sliceString(0, insert.length, "\n");

        // There're not early return because replacing text
        // generate a delete and insert
        if (fromA != toA) {
          accumulated_changes.push(OtOperation.remove(fromA, toA));
        }

        if (insert.length !== 0) {
          accumulated_changes.push(OtOperation.insert(fromA, content));
        }
      },
    );

    cb = setTimeout(() => realEventHandler(update), 100);
  };
}

let sended_ops: OtOperation[] = [];

function sendableOps(ops: OtOperation[]): OtOperation[] {
  let diff_idx = 0;

  for (let idx = 0; idx < sended_ops.length; idx++) {
    const op = ops[diff_idx];
    const sended_op = sended_ops[idx];

    if (OtOperation.equal(op, sended_op)) {
      diff_idx++;
    } else {
      diff_idx = 0;
    }
  }

  ops = ops.slice(diff_idx);

  sended_ops = sended_ops.concat(ops);

  ops = optimizeOps(ops);

  return ops;
}

function receiveOps(
  editor: EditorView,
  file: string,
  msg: ServerMessage<ServerMessageKind.Sync>,
) {
  const owner = untrack(wsSessionId);
  const local_revision = editingFiles[file].synced_revision;
  const desyncronized_history = msg.actions.slice(local_revision);
  const me = untrack(wsSessionId);

  setEditingFiles(file, "synced_revision", msg.revision);

  const initialCursors = editor.state.selection.ranges;
  const mainIndex = editor.state.selection.mainIndex;

  for (const action of desyncronized_history) {
    if (action.owner === me) {
      continue;
    }

    const annotations = [ownerAnnotation.of(action.owner)];

    if (action.kind === OtOperationKind.Insert) {
      editor.dispatch({
        annotations,
        changes: { from: action.from, insert: action.text },
      });
    } else {
      editor.dispatch({
        annotations,
        changes: { from: action.from, to: action.to },
      });
    }
  }

  const newCursors = initialCursors.map((v) =>
    EditorSelection.range(
      transformIndex(v.anchor, owner, desyncronized_history),
      transformIndex(v.head, owner, desyncronized_history),
    )
  );

  editor.update([
    editor.state.update({
      selection: EditorSelection.create(newCursors, mainIndex),
    }),
  ]);

  setSyncFiles(file, editor.state.doc.toString());
}
