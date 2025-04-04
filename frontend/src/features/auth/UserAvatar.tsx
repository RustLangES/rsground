import { createMemo, Show } from "solid-js";

import { authInfo } from "./store";

import styles from "./UserAvatar.module.sass";
import { SolidUserIcon } from "../../icons/SolidUser";

export function RawUserAvatar() {
  const has_avatar = createMemo(() => !!authInfo()?.avatar_url);

  return (
    <>
      <Show when={has_avatar()}>
        <img class={styles.raw_avatar_img} src={authInfo()?.avatar_url} />
      </Show>
      <Show when={!has_avatar()}>
        <SolidUserIcon height="100%" width="100%" />
      </Show>
    </>
  );
}
