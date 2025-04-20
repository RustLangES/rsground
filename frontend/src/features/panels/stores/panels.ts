import { DockviewApi } from "dockview-core";
import { createSignal } from "solid-js";

export const [dockview, setDockview] = createSignal<DockviewApi>(null, { name: "globalDockview" });
