import { CodeMirror } from "@solid-codemirror/codemirror";

import { getNodeByPath } from "@features/file-explorer/stores";
import { FileNodeKind } from "@features/file-explorer/types";

import { rustExtensions, syncExtension, syncExtensionListener } from "../utils";
import { syncFiles } from "../stores";

import styles from "./CodeEditor.module.sass";

export interface CodeEditorProps {
  /** full-path of the target file to edit */
  file: string;
}

export function CodeEditor(props: CodeEditorProps) {
  const [file, _] = getNodeByPath(props.file);

  if (file.kind == FileNodeKind.Folder) {
    throw new Error("Really?? Edit a folder?");
  }

  return (
    <CodeMirror
      class={styles.container}
      value={syncFiles[props.file]}
      extensions={[
        ...rustExtensions(styles),
        syncExtension(file.data),
      ]}
      onEditorMount={(editor) => {
        editor.setTabFocusMode(true);
        syncExtensionListener(editor, file.data.fullPath);
      }}
    />
  );
}
