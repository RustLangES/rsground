import { CodeMirror } from "@solid-codemirror/codemirror";
import { collab, sendableUpdates } from "@codemirror/collab";

import { getNodeByPath } from "@features/file-explorer/stores";

import { rustExtensions } from "../utils";
import { syncFiles } from "../stores";

import styles from "./CodeEditor.module.sass";
import { OtOperation, OtOperationKind } from "../types";

export interface CodeEditorProps {
  /** full-path of the target file to edit */
  file: string;
}

export function CodeEditor(props: CodeEditorProps) {
  const [file, setFile] = getNodeByPath(props.file);

  return (
    <CodeMirror
      class={styles.container}
      value={syncFiles[props.file]}
      extensions={[
        ...rustExtensions(styles),
        collab(),
      ]}
      onEditorMount={(editor) => {
        editor.setTabFocusMode(true);

        editor.dom.addEventListener("keydown", () => {
          console.log(
            sendableUpdates(editor.state).map((update) => {
              let acc = [];
              update.changes.iterChanges((fromA, toA, _fromB, _toB, insert) => {
                if (fromA == toA) {
                  acc.push(
                    {
                      kind: OtOperationKind.Insert,
                      owner: "me",
                      from: fromA,
                      content: insert.sliceString(0, insert.length, "\n"),
                    } satisfies OtOperation,
                  );
                } else {
                  acc.push(
                    {
                      kind: OtOperationKind.Delete,
                      owner: "me",
                      from: fromA,
                      to: toA,
                    } satisfies OtOperation,
                  );
                }
              });
              return acc;
            }),
          );
        });
      }}
    />
  );
}
