import { createStore } from "solid-js/store";
import { Cursor, EditingFilesStore } from "../types";

/** Key-value of files and their action history */
export const [editingFiles, setEditingFiles] = createStore<EditingFilesStore>({});

/** Key-value of files and the cursors of each user*/
export const [cursorsFiles, setCursorsFiles] = createStore<Record<string, Record<string, Cursor[]>>>({});
