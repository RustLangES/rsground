import { EditorView } from "codemirror";
import { EditorSelection } from "@codemirror/state";
import { sendableUpdates, Update } from "@codemirror/collab";

import { FileNode } from "@features/file-explorer/types";
import { sendMessage } from "@features/ws/services";
import { ClientMessageKind } from "@features/ws/types";
import { sameValueRecord } from "@utils/sameValueRecord";

import { OtOperation } from "../types";
import { optimizeOps } from "./optimizeOps";

export function syncExtension(file: FileNode) {
  return EditorView.domEventObservers(
    sameValueRecord([
      "blur",
      "focus",
      "keyup",
      "keydown",
      "mouseup",
      "mousedown",
      "mousemove",
      "mouseleave",
      "mouseenter",
    ], anyEventHandler(file)),
  );
}

let expected_revision = 0;

function anyEventHandler(file: FileNode) {
  let lastCursors: EditorSelection;
  const handleCursor = (editor: EditorView) => {
    if (lastCursors && editor.state.selection.eq(lastCursors)) return;
    lastCursors = editor.state.selection;

    let cursors = editor.state.selection.ranges.map((value) => ({
      from: value.from,
      to: value.to,
      head: value.head,
    }));

    console.log("CURSORS", cursors);
  };

  let lastEvents = [];
  const handleOps = (events: readonly Update[]) => {
    if (lastEvents.length === events.length) return;
    lastEvents = events as any[];

    let ops: OtOperation[] = [];

    events.forEach((update) => {
      update.changes.iterChanges(
        (fromA, toA, _fromB, _toB, insert) => {
          const content = insert.sliceString(0, insert.length, "\n");

          // There're not early return because replacing text
          // generate a delete and insert
          if (fromA != toA) {
            ops.push(OtOperation.remove(fromA, toA));
          }

          if (insert.length !== 0) {
            ops.push(OtOperation.insert(fromA, content));
          }
        },
      );
    });

    console.log("PRE OPS", ops);

    ops = sendableOps(ops);

    console.log("OPS", ops);

    console.log("REVISION", expected_revision);

    sendMessage(ClientMessageKind.Sync, {
      file: file.fullPath,
      revision: expected_revision,
      actions: ops,
    });

    expected_revision += ops.length;
  };

  const realEventHandler = (editor: EditorView) => {
    let events = sendableUpdates(editor.state);

    handleCursor(editor);
    handleOps(events);
  };

  let cb: NodeJS.Timeout;
  return (_: unknown, editor: EditorView) => {
    if (cb) clearTimeout(cb);

    cb = setTimeout(() => realEventHandler(editor), 100);
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
