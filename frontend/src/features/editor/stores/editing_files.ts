import { createStore } from "solid-js/store";
import { EditingFilesStore } from "../types";

/** Key-value of files and their action history */
export const [editingFiles, setEditingFiles] = createStore<EditingFilesStore>();
