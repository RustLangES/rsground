import { EditorView } from "codemirror";
import { Annotation } from "@codemirror/state";
import { OpSeq } from "frontend-wasm";

import { unicodeLength } from "./unicodeLength";

export const syncAnnotationType = Annotation.define<string>();
export const syncAnnotation = syncAnnotationType.of("SYNC");

export function applyOperation(
  operation: OpSeq,
  insertion: (from: number, content: string) => void,
  deletion: (from: number, to: number) => void,
): boolean {
  if (operation.is_noop()) return false;

  const ops: (string | number)[] = JSON.parse(operation.to_string());
  let index = 0;

  for (const op of ops) {
    if (typeof op === "string") {
      // Insert
      insertion(index, op);

      index += unicodeLength(op);
    } else if (op >= 0) {
      // Retain
      index += op;
    } else {
      // Delete
      const chars = -op;
      deletion(index, index + chars);
    }
  }

  return true;
}

export function applyOperationToString(
  text: string,
  operation: OpSeq,
): string {
  applyOperation(
    operation,
    (from, insert) => text = text.slice(0, from) + insert + text.slice(from),
    (from, to) => text = text.slice(0, from) + text.slice(to),
  );

  return text;
}

export function applyOperationToView(
  view: EditorView,
  operation: OpSeq,
) {
  applyOperation(
    operation,
    (from, insert) =>
      view.dispatch({
        annotations: [syncAnnotation],
        changes: { from, insert },
      }),
    (from, to) =>
      view.dispatch({
        annotations: [syncAnnotation],
        changes: { from, to },
      }),
  );
}
