import { setIsAuthOpen } from "@features/auth/stores";
import { AuthModal, RawUserAvatar } from "@features/auth/views";
import { setIsColabOpen } from "@features/colab/stores";
import { Colab } from "@features/colab/views";
import { FileExplorer } from "@features/file-explorer/views";
import { ContextMenu } from "@features/context-menu/views";
import { ThemeSelector } from "@features/theme/views";
import { MenuIcon } from "@icons/Menu";
import { ChevronLeftIcon } from "@icons/ChevronLeft";
import { DocumentIcon } from "@icons/Document";
import { ShareNodesIcon } from "@icons/ShareNodes";

import { isSidebarOpen, setIsSidebarOpen } from "../stores";
import { SidebarNavItem } from "./SidebarNavItem";

import styles from "./Sidebar.module.sass";

export function Sidebar() {
  return (
    <div
      class={styles.container}
      attr:data-closed={!isSidebarOpen() || null}
      attr:aria-hidden={!isSidebarOpen() || null}
    >
      <nav class={styles.nav}>
        <ul class={styles.nav_items}>
          <ContextMenu
            as={SidebarNavItem}
            tooltip="Menu"
            useLeftClick
            useRightClick={false}
            followCursor={false}
            options={{
              Theme: <ThemeSelector />,
            }}
          >
            <MenuIcon />
          </ContextMenu>

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
            <ShareNodesIcon />
            <Colab />
          </SidebarNavItem>

          <SidebarNavItem>
            <DocumentIcon />
          </SidebarNavItem>

          <SidebarNavItem
            tooltip={isSidebarOpen() ? "Close" : "Open"}
            onClick={() => setIsSidebarOpen((prev) => !prev)}
          >
            <ChevronLeftIcon />
          </SidebarNavItem>
        </ul>
      </nav>

      <div class={styles.body}>
        <FileExplorer />
      </div>
    </div>
  );
}
