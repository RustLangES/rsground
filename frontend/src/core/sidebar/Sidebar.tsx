import { ParentProps } from "solid-js";
import Tooltip from "@corvu/tooltip";
import { FaSolidShareNodes, FaSolidUser } from "solid-icons/fa";

import { AuthModal, setIsAuthOpen } from "../auth";
import { Colab, setIsColabOpen } from "../colab";
import { FileExplorer } from "./file-explorer";
import { isSidebarOpen, setIsSidebarOpen } from "./store";

import style from "./Sidebar.module.sass";
import { RawUserAvatar } from "../auth/UserAvatar";

interface SidebarNavItemProps {
  /**
   * Don't add padding in the button, perfect for "background" images
   */
  fullSized?: boolean;

  tooltip?: string;

  onClick?: () => void;
}

function SidebarNavItem(props: ParentProps<SidebarNavItemProps>) {
  if (props.tooltip) {
    return (
      <Tooltip placement="bottom" openDelay={200} hoverableContent={false}>
        <Tooltip.Trigger
          as="button"
          classList={{
            [style.nav_item]: true,
            [style.nav_item_full]: props.fullSized
          }}
          onClick={props.onClick}
        >
          {props.children}
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content class={style.tooltip}>
            {props.tooltip}
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip>
    );
  } else {
    return (
      <li class={style.nav_item}>
        {props.children}
      </li>
    );
  }
}

export function Sidebar() {
  return (
    <div
      classList={{ [style.container]: true, [style.closed]: !isSidebarOpen() }}
    >
      <nav class={style.nav}>
        <ul class={style.nav_items}>
          <SidebarNavItem tooltip="Menu">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 20 20"
            >
              <path
                d="M4 5H16a1 1 0 010 2H4a1 1 0 010-2Zm0 4H16a1 1 0 010 2H4a1 1 0 010-2Zm0 4H16a1 1 0 010 2H4a1 1 0 010-2Z"
                fill="currentColor"
              />
            </svg>
          </SidebarNavItem>

          <SidebarNavItem
            fullSized
            tooltip="Auth"
            onClick={() => setIsAuthOpen(true)}
          >
            <AuthModal>
              <RawUserAvatar />
            </AuthModal>
          </SidebarNavItem>

          <SidebarNavItem tooltip="Colab" onClick={() => setIsColabOpen(true)}>
            <FaSolidShareNodes />
            <Colab />
          </SidebarNavItem>

          <SidebarNavItem>
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M7 2H15L20 6V19C20 19.7956 19.6839 20.5587 19.1213 21.1213C18.5587 21.6839 17.7956 22 17 22H7C6.20435 22 5.44129 21.6839 4.87868 21.1213C4.31607 20.5587 4 19.7956 4 19V5C4 4.20435 4.31607 3.44129 4.87868 2.87868C5.44129 2.31607 6.20435 2 7 2Z"
                fill="currentColor"
              />
            </svg>
          </SidebarNavItem>

          <SidebarNavItem
            tooltip={isSidebarOpen() ? "Close" : "Open"}
            onClick={() => setIsSidebarOpen((prev) => !prev)}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M14 7L9 12L14 17"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </SidebarNavItem>
        </ul>
      </nav>

      <div class={style.body}>
        <FileExplorer />
      </div>
    </div>
  );
}
