import { FaSolidShareNodes } from "solid-icons/fa";

import { MenuIcon } from "@icons/Menu";
import { ChevronLeft } from "@icons/ChevronLeft";
import { DocumentIcon } from "@icons/Document";
import { setIsAuthOpen } from "@features/auth/stores";
import { AuthModal, RawUserAvatar } from "@features/auth/views";
import { setIsColabOpen } from "@features/colab/stores";
import { Colab } from "@features/colab/views";
import { FileExplorer } from "@features/file-explorer/views";

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
          <SidebarNavItem tooltip="Menu">
            <MenuIcon />
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
            <DocumentIcon />
          </SidebarNavItem>

          <SidebarNavItem
            tooltip={isSidebarOpen() ? "Close" : "Open"}
            onClick={() => setIsSidebarOpen((prev) => !prev)}
          >
            <ChevronLeft />
          </SidebarNavItem>
        </ul>
      </nav>

      <div class={styles.body}>
        <FileExplorer />
      </div>
    </div>
  );
}
