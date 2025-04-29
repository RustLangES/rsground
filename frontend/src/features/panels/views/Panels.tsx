import { getOwner, runWithOwner } from "solid-js";
import {
  DockviewComponent,
  DockviewTheme,
  IContentRenderer,
} from "dockview-core";

import { setDockview } from "../stores";
import { CodePanel } from "./CodePanel";
import { OutputPanel } from "./OutputPanel";

import "dockview-core/dist/styles/dockview.css";
import "./dockview.sass";

import styles from "./Panels.module.sass";

export function Panels() {
  const owner = getOwner();
  const element = <div class={styles.container} /> as HTMLElement;

  const dockview = new DockviewComponent(element, {
    theme: {
      name: "rsground",
      className: "rsground-dockview",
      gap: 10,
      dndOverlayMounting: "absolute",
      dndPanelOverlay: "group",
    } satisfies DockviewTheme,
    disableFloatingGroups: true,
    singleTabMode: "default",

    createComponent(options) {
      const element = (options.name == "code"
        ? runWithOwner(owner, () => CodePanel(options.id))
        : options.name == "output"
        ? OutputPanel()
        : <span>Esto es canallesco</span>) as HTMLElement;

      return {
        element,
        init(_params) {},
      } satisfies IContentRenderer;
    },
  });

  dockview.api.onDidRemovePanel((e) => {
    if (e.id == "output") {
      dockview.api.addPanel({
        id: "output",
        component: "output",
        title: "Output",
        initialHeight: 20,
        minimumHeight: 50,
        position: { direction: "below" },
      });
    }
  });

  setDockview(dockview.api);

  dockview.api.addPanel({
    id: "output",
    component: "output",
    title: "Output",
    initialHeight: 20,
    minimumHeight: 50,
    position: { direction: "below" },
  });

  return element;
}
