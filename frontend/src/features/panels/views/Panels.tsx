import "dockview-core/dist/styles/dockview.css";

import {
  DockviewComponent,
  DockviewTheme,
  IContentRenderer,
} from "dockview-core";

import { CodePanel } from "./CodePanel";
import { OutputPanel } from "./OutputPanel";

import "./dockview.sass";
import styles from "./Panels.module.sass";

export function Panels() {
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
        ? CodePanel()
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
  })

  dockview.api.addPanel({
    id: "file:main.rs",
    component: "code",
    title: "main.rs",
  });

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
