import { createStore } from "solid-js/store";

export const [contextMenus, updateContextMenus] = createStore<boolean[]>([], {
  name: "ContextMenus",
});

export function addContextMenu(): number {
  const id = contextMenus.length;

  updateContextMenus(id, false)

  return id;
}

export function openContextMenu(id: number) {
  updateContextMenus(id, true)
}

export function setContextMenu(id: number, state: boolean) {
  updateContextMenus(id, state)
}

export function closeAllContextMenus() {
  // All the `true` state are updated to `false`
  updateContextMenus((state) => state, false)
}
