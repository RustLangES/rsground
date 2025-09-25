import { Show } from "solid-js";
import { Spinner } from "@components/Spinner";
import styles from "./LoadingLsp.module.sass";
import { loadingLsp } from "../stores";

export function LoadingLsp() {
  return (
    <Show when={loadingLsp.isLoading}>
      <div class={styles.container}>
        <span class={styles.message}>
          {loadingLsp.message}
        </span>
        <span class={styles.title}>
          {loadingLsp.title}
          <Show when={loadingLsp.percentage}>
            {v => ` ${v()}%`}
          </Show>
          <Spinner />
        </span>
      </div>
    </Show>
  );
}
