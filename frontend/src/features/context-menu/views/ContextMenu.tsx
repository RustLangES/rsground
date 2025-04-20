import Popover, { DynamicProps } from "@corvu/popover";
import { For, JSX, ParentProps, splitProps, ValidComponent } from "solid-js";
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
    { level?: string; disabled?: boolean; onClick?: () => void } | JSX.Element
  >;

  /**
   * Open dialog when user clicks with left click
   * @defaultvalue false
   */
  useLeftClick?: boolean;

  /**
   * Open dialog when user clicks with right click
   * @defaultvalue true
   */
  useRightClick?: boolean;

  /**
   * Spawn dialog on cursor position
   * @defaultValue true
   */
  followCursor?: boolean;
}

export function ContextMenu(
  props_: DynamicProps<
    ValidComponent,
    ParentProps<ContextMenuProps>
  >,
) {
  const [props, restProps] = splitProps(props_, [
    "as",
    "children",
    "options",
    "useLeftClick",
    "useRightClick",
    "followCursor",
  ]);

  const contextMenuId = addContextMenu();
  let anchorRef!: HTMLElement;

  const openOnMouseEvent = (ev: MouseEvent) => {
    ev.preventDefault();
    ev.stopPropagation();

    closeAllContextMenus();

    // Align context menu arrow with cursor event
    if (props.followCursor != false) {
      anchorRef.style.top = `${ev.clientY - 16}px`;
      anchorRef.style.left = `${ev.clientX + 8}px`;
    }
    openContextMenu(contextMenuId);
  };

  return (
    <Popover
      open={contextMenus[contextMenuId]}
      onOpenChange={setContextMenu.bind(null, contextMenuId)}
      placement="right-start"
      closeOnEscapeKeyDown
      closeOnOutsidePointer
      closeOnOutsideFocus={false}
      trapFocus={true}
    >
      <Dynamic
        {...restProps}
        component={props.as}
        onClick={(ev: MouseEvent) => {
          restProps.onClick?.(ev);
          props.useLeftClick == true && openOnMouseEvent(ev);
        }}
        onContextMenu={props.useRightClick != false && openOnMouseEvent}
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
              // This excludes JSX element from object defined item
              typeof item !== "object" || item instanceof Array ||
                item instanceof Node
                ? item
                : (
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
                )
            )}
          </For>
        </Popover.Content>
      </Popover.Portal>
    </Popover>
  );
}
