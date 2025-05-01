import { observable, onCleanup } from "solid-js";
import { CodeMirror } from "@solid-codemirror/codemirror";
import { Compartment, EditorState } from "@codemirror/state";

import { projectAccess } from "@features/colab/stores";
import { getNodeByPath } from "@features/file-explorer/stores";
import { FileNodeKind } from "@features/file-explorer/types";
import { AccessLevel } from "@features/ws/types";

import { rustExtensions, syncExtension, syncExtensionListener } from "../utils";
import { setEditingFiles, syncFiles } from "../stores";

import styles from "./CodeEditor.module.sass";

export interface CodeEditorProps {
  /** full-path of the target file to edit */
  file: string;
}

export function CodeEditor(props: CodeEditorProps) {
  const [file, _] = getNodeByPath(props.file);
  const readOnly = new Compartment();

  if (file.kind == FileNodeKind.Folder) {
    throw new Error("Really?? Edit a folder?");
  }

  setEditingFiles(file.data.fullPath, "editor_open", true);

  onCleanup(() => {
    setEditingFiles(file.data.fullPath, "editor_open", false);
  });

  return (
    <CodeMirror
      class={styles.container}
      value={syncFiles[props.file]}
      extensions={[
        ...rustExtensions(styles),
        readOnly.of([]),
        syncExtension(file.data),
      ]}
      onEditorMount={(editor) => {
        editor.setTabFocusMode(true);
        syncExtensionListener(editor, file.data.fullPath);

        observable(projectAccess).subscribe((access) => {
          editor.dispatch({
            effects: readOnly.reconfigure(
              EditorState.readOnly.of(access !== AccessLevel.Editor),
            ),
          });
        });
      }}
    />
  );
}
