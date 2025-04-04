import Popover from "@corvu/popover";
import { ParentProps, Show } from "solid-js";
import { FaBrandsGithub } from "solid-icons/fa";

import { TextField } from "../../components/TextField";
import { SolidUserIcon } from "../../icons/SolidUser";

import {
  authInfo,
  isAuthOpen,
  isGithubLogged,
  isLoadingAuthInfo,
  logout,
  setIsAuthOpen,
} from "./store";
import { RawUserAvatar } from "./UserAvatar";

import styles from "./AuthModal.module.sass";
import { loginGithub } from "./service";
import { Spinner } from "../../components/Spinner";

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
          </Show>

          <Show when={!isGithubLogged() && !isLoadingAuthInfo()}>
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
          </Show>
        </Popover.Content>
      </Popover.Portal>
    </Popover>
  );
}
