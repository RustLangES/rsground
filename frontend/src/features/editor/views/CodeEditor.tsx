import {
  batch,
  createEffect,
  createSignal,
  observable,
  onCleanup,
  untrack,
} from "solid-js";
import { CodeMirror } from "@solid-codemirror/codemirror";
import { EditorView } from "codemirror";
import { Decoration } from "@codemirror/view";
import { Compartment, EditorState } from "@codemirror/state";

import { authInfo } from "@features/auth/stores";
import { projectAccess } from "@features/colab/stores";
import { getNodeByPath } from "@features/file-explorer/stores";
import { FileNodeKind } from "@features/file-explorer/types";
import { AccessLevel, ServerMessageKind } from "@features/ws/types";
import { onWsMessage } from "@features/ws/services";

import { rustExtensions, syncExtension, syncExtensionListener } from "../utils";
import {
  cursorsFiles,
  setCursorsFiles,
  setEditingFiles,
  syncFiles,
} from "../stores";
import { Cursor } from "../types";

import styles from "./CodeEditor.module.sass";

export interface CodeEditorProps {
  /** full-path of the target file to edit */
  file: string;
}

export function CodeEditor(props: CodeEditorProps) {
  const [file, _] = getNodeByPath(props.file);
  const file_path = file.data.fullPath;
  const [editor, setEditor] = createSignal<EditorView>();
  const readOnly = new Compartment();
  const cursors = new Compartment();

  if (file.kind == FileNodeKind.Folder) {
    throw new Error("Really?? Edit a folder?");
  }

  setEditingFiles(file.data.fullPath, "editor_open", true);

  onCleanup(() => {
    setEditingFiles(file.data.fullPath, "editor_open", false);
  });

  onWsMessage(ServerMessageKind.SyncCursors, (msg) => {
    if (msg.file !== file_path) return;

    batch(() => {
      for (const [user, cursors] of Object.entries(msg.cursors)) {
        setCursorsFiles(file_path, user, cursors.map(Cursor.from));
      }
    });
  });

  createEffect(() => {
    if (!editor()) return;

    const stored_cursors = cursorsFiles[file.data.fullPath];

    let collected_cursors = [];
    const max_length = editor().state.doc.length;

    for (const [user, user_cursors] of Object.entries(stored_cursors)) {
      if (user === untrack(authInfo)?.id) continue;

      for (const cursor of user_cursors) {
        collected_cursors.push(Cursor.toDecoration(
          {
            from: Math.min(cursor.from, max_length),
            to: Math.min(cursor.to, max_length),
          },
          user,
          styles,
        ));

        collected_cursors.push(
          Cursor.toDecoration(
            {
              from: Math.min(cursor.to, max_length),
              to: Math.min(cursor.to, max_length),
            },
            user,
            styles,
          ),
        );
      }
    }

    const decorations = EditorView.decorations.of(
      Decoration.set(collected_cursors, true),
    );
    editor().dispatch({ effects: cursors.reconfigure(decorations) });
  });

  return (
    <CodeMirror
      class={styles.container}
      value={syncFiles[props.file]}
      extensions={[
        ...rustExtensions(styles),
        readOnly.of([]),
        cursors.of(EditorView.decorations.of(Decoration.set([]))),
        syncExtension(file.data),
      ]}
      onEditorMount={(editor) => {
        setEditor(editor);
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
