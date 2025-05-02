import { batch } from "solid-js";
import { unwrap } from "solid-js/store";

import { setCursorsFiles, setEditingFiles, setSyncFiles } from "@features/editor/stores";
import { sendMessage } from "@features/ws/services";
import { ClientMessageKind, DocumentInfo } from "@features/ws/types";

import { fileExplorer, setFileExplorer } from "../stores";
import { mergeFiles } from "../utils";

export function createNewFile(file: string) {
  sendMessage(ClientMessageKind.FileCreate, { file });
}

export function syncFiles(files: Record<string, DocumentInfo>) {
  batch(() => {
    for (const [path, doc] of Object.entries(files)) {
      setSyncFiles(path, doc.text);
      setEditingFiles(path, {
        synced_revision: doc.revision,
        editor_open: false,
      });
      setCursorsFiles(path, {});
    }

    const nodes = mergeFiles(Object.keys(files), unwrap(fileExplorer).nodes);

    setFileExplorer("nodes", nodes);
  });
}
