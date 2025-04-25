import { EditorView } from "codemirror";
import { EditorSelection } from "@codemirror/state";
import { sendableUpdates } from "@codemirror/collab";

import { FileNode } from "@features/file-explorer/types";
import { sameValueRecord } from "@utils/sameValueRecord";

import { OtOperation } from "../types";

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

function anyEventHandler(file: FileNode) {
  let lastEvents = [];
  let lastCursors: EditorSelection;
  const realEventHandler = (editor: EditorView) => {
    let events = sendableUpdates(editor.state);

    if (
      lastEvents.length === events.length && lastCursors &&
      editor.state.selection.eq(lastCursors)
    ) return;
    lastEvents = events as any[];
    lastCursors = editor.state.selection;

    let acc = [];

    events.forEach((update) => {
      update.changes.iterChanges(
        (fromA, toA, fromB, toB, insert) => {
          const content = insert.sliceString(0, insert.length, "\n");

          // There're not early return because replacing text
          // generate a delete and insert
          if (fromB != toB) {
            acc.push(OtOperation.remove(fromA, toA));
          }

          if (insert.length !== 0) {
            acc.push(OtOperation.insert(fromA, content));
          }
        },
      );
    });

    console.log(editor.state.selection);
    console.log(acc);
  };

  let cb: NodeJS.Timeout;
  return (_: unknown, editor: EditorView) => {
    if (cb) clearTimeout(cb);

    cb = setTimeout(() => realEventHandler(editor), 20);
  };
}
