import { createMemo, Show } from "solid-js";

import { authInfo, guestAvatar } from "../stores";

import styles from "./UserAvatar.module.sass";

export function RawUserAvatar() {
  const has_avatar = createMemo(() => !!authInfo()?.avatar_url);

  return (
    <>
      <Show when={has_avatar()}>
        <img class={styles.raw_avatar_img} src={authInfo()?.avatar_url} />
      </Show>
      <Show when={!has_avatar()}>
        <img class={styles.guest_avatar_img} src={guestAvatar()} />
      </Show>
    </>
  );
}
