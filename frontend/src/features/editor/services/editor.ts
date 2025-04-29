import { untrack } from "solid-js";
import { dockview } from "@features/panels/stores";

export async function openFile(filepath: string) {
  const id = `file:${filepath}`;
  const filename = filepath.split("/").pop();

  if (!untrack(dockview).getPanel(id)) {
    untrack(dockview).addPanel({
      id,
      component: "code",
      title: filename,
      position: { direction: "above", referencePanel: "output" },
    });
  }
}
