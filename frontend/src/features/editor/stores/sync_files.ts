import { createStore } from "solid-js/store";
import { SyncFilesStore } from "../types";

export const [syncFiles, setSyncFiles] = createStore<SyncFilesStore>();
