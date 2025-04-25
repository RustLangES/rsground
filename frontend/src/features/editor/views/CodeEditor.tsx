import { CodeMirror } from "@solid-codemirror/codemirror";
import { collab } from "@codemirror/collab";

import { getNodeByPath } from "@features/file-explorer/stores";
import { wsSessionId } from "@features/ws/stores";

import { rustExtensions, syncExtension } from "../utils";
import { syncFiles } from "../stores";

import styles from "./CodeEditor.module.sass";
import { FileNodeKind } from "@features/file-explorer/types";

export interface CodeEditorProps {
  /** full-path of the target file to edit */
  file: string;
}

export function CodeEditor(props: CodeEditorProps) {
  const [file, setFile] = getNodeByPath(props.file);

  if (file.kind == FileNodeKind.Folder) {
    throw new Error("Really?? Edit a folder?")
  }

  return (
    <CodeMirror
      class={styles.container}
      value={syncFiles[props.file]}
      extensions={[
        ...rustExtensions(styles),
        collab({ clientID: wsSessionId() }),
        syncExtension(file.data)
      ]}
      onEditorMount={(editor) => {
        editor.setTabFocusMode(true);
      }}
    />
  );
}
