import Popover, { DynamicProps } from "@corvu/popover";
import { For, ParentProps, splitProps, ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";

import {
  addContextMenu,
  closeAllContextMenus,
  contextMenus,
  openContextMenu,
  setContextMenu,
} from "../stores";

import styles from "./ContextMenu.module.sass";

export interface ContextMenuProps {
  options: Record<
    string,
    { level?: string; disabled?: boolean; onClick?: () => void }
  >;
}

export function ContextMenu(
  props_: DynamicProps<
    ValidComponent,
    ParentProps<ContextMenuProps>
  >,
) {
  const [props, restProps] = splitProps(props_, ["as", "children", "options"]);

  const contextMenuId = addContextMenu();
  let anchorRef!: HTMLElement;

  return (
    <Popover
      open={contextMenus[contextMenuId]}
      onOpenChange={setContextMenu.bind(null, contextMenuId)}
      placement="right-start"
      closeOnEscapeKeyDown
      closeOnOutsidePointer
      closeOnOutsideFocus={false}
      trapFocus={false}
    >
      <Dynamic
        {...restProps}
        component={props.as}
        onContextMenu={(ev: MouseEvent) => {
          ev.preventDefault();
          ev.stopPropagation();

          closeAllContextMenus();

          // Align context menu arrow with cursor event
          anchorRef.style.top = `${ev.clientY - 16}px`;
          anchorRef.style.left = `${ev.clientX + 8}px`;
          openContextMenu(contextMenuId);
        }}
      >
        <Popover.Anchor
          class={styles.anchor}
          ref={(r) => anchorRef = r}
        />

        {props.children}
      </Dynamic>

      <Popover.Portal>
        <Popover.Content
          as="ul"
          class={styles.content}
        >
          <For each={Object.entries(props.options)}>
            {([name, item]) => (
              <li
                tabindex="1"
                classList={{
                  [styles.disabled]: item.disabled,

                  [styles.item]: !["error", "warning"].includes(item.level),
                  [styles.item_error]: item.level == "error",
                  [styles.item_warning]: item.level == "warning",
                }}
                onClick={item.onClick}
              >
                {name}
              </li>
            )}
          </For>
        </Popover.Content>
      </Popover.Portal>
    </Popover>
  );
}
