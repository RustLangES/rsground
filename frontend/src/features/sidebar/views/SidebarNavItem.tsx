import Tooltip from "@corvu/tooltip";
import { ParentProps } from "solid-js";

import styles from "./SidebarNavItem.module.sass"

export interface SidebarNavItemProps {
  /**
   * Don't add padding in the button, perfect for "background" images
   */
  fullSized?: boolean;

  tooltip?: string;

  onClick?: () => void;
}

export function SidebarNavItem(props: ParentProps<SidebarNavItemProps>) {
  if (props.tooltip) {
    return (
      <Tooltip placement="bottom" openDelay={200} hoverableContent={false}>
        <Tooltip.Trigger
          as="button"
          classList={{
            [styles.nav_item]: true,
            [styles.nav_item_full]: props.fullSized,
          }}
          onClick={props.onClick}
        >
          {props.children}
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content class={styles.tooltip}>
            {props.tooltip}
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip>
    );
  } else {
    return (
      <li class={styles.nav_item}>
        {props.children}
      </li>
    );
  }
}
