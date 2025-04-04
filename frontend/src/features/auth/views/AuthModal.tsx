import Popover from "@corvu/popover";
import { ParentProps, Show } from "solid-js";
import { FaBrandsGithub } from "solid-icons/fa";
import { Spinner } from "@components/Spinner";
import { TextField } from "@components/TextField";
import { SolidUserIcon } from "@icons/SolidUser";
import {
  authInfo,
  isAuthOpen,
  isGithubLogged,
  isLoadingAuthInfo,
  setIsAuthOpen,
} from "../stores";
import { loginGithub, logout } from "../services";
import { RawUserAvatar } from "./UserAvatar";

import styles from "./AuthModal.module.sass";

export function AuthModal(props: ParentProps) {
  return (
    <Popover
      open={isAuthOpen()}
      onOpenChange={setIsAuthOpen}
      placement="bottom-start"
    >
      {props.children}

      <Popover.Anchor />

      <Popover.Portal>
        <Popover.Content
          classList={{
            [styles.base]: true,
            [styles.content_h]: isGithubLogged(),
            [styles.content_v]: !isGithubLogged(),
          }}
        >
          <Show when={isLoadingAuthInfo()}>
            <Spinner />
          </Show>

          <Show when={isGithubLogged() && !isLoadingAuthInfo()}>
            <AuthModalGithub />
          </Show>

          <Show when={!isGithubLogged() && !isLoadingAuthInfo()}>
            <AuthModalGuest />
          </Show>
        </Popover.Content>
      </Popover.Portal>
    </Popover>
  );
}

function AuthModalGithub() {
  return (
    <>
      <div class={styles.user_avatar}>
        <RawUserAvatar />
      </div>

      <div class={styles.username_container}>
        <span>Logged as</span>

        <h3 class={styles.username}>
          {authInfo()?.name ?? authInfo()?.username ?? "Guest"}
        </h3>
      </div>

      <button
        class={styles.logout}
        onClick={() => {
          logout();
        }}
      >
        Log out
      </button>
    </>
  );
}

function AuthModalGuest() {
  return (
    <>
      <TextField
        value={authInfo()?.name ?? "Guest"}
        beforeIcon={<SolidUserIcon />}
      />

      <button
        class={styles.login_github}
        onClick={() => {
          loginGithub();
        }}
      >
        <FaBrandsGithub size="1.2rem" />
        Login with Github
      </button>
    </>
  );
}
