import Dialog from "@corvu/dialog";
import { FaBrandsGithub, FaSolidLock } from "solid-icons/fa";
import { For } from "solid-js";

import { TextField } from "../../components/TextField";
import { Switchbox } from "../../components/Switchbox";
import { SelectField } from "../../components/SelectField";

import { isColabOpen, setIsColabOpen } from "./store";

import styles from "./Colab.module.sass";

export function Colab() {
  const requestUsers = ["CHIWO", "Jopzgo", "gg0074x", "Otro"];

  return (
    <Dialog open={isColabOpen()} onOpenChange={setIsColabOpen}>
      <Dialog.Portal>
        <Dialog.Overlay class={styles.overlay} />
        <Dialog.Content class={styles.content}>
          <h2 class={styles.title}>Live Collaboration</h2>

          <div class={styles.container}>
            <div>
              <h3 class={styles.subtitle}>Room settings</h3>

              <label class={styles.checkbox_input}>
                Public room
                <Switchbox />
              </label>

              <TextField
                beforeIcon={<FaSolidLock />}
                placeholder="Leave empty for no password"
              />

              <div class={styles.buttons_container}>
                <button>Copy colab link</button>
                <button>Copy fork link</button>
                <button>Fork</button>
              </div>
            </div>

            <div>
              <h3 class={styles.subtitle}>Members</h3>

              <TextField
                beforeIcon={<FaBrandsGithub />}
                placeholder="Username"
              />

              <ul class={styles.user_list}>
                <For each={requestUsers}>
                  {(name, idx) => (
                    <li class={styles.member}>
                      <span class={styles.member_name}>
                        {name}
                      </span>

                      <SelectField
                        value={idx() % 2 == 0 ? "Editor" : "Viewer"}
                        options={["Editor", "Viewer"]}
                      />
                    </li>
                  )}
                </For>
              </ul>

              <h3 class={styles.subtitle}>Pending Requests</h3>
              <ul class={styles.user_list}>
                <For each={requestUsers}>
                  {(name) => (
                    <li class={styles.member}>
                      <span class={styles.member_name}>
                        {name}
                      </span>

                      <ul class={styles.button_group}>
                        <button class={styles.success}>Allow</button>
                        <button class={styles.error}>Kick</button>
                      </ul>
                    </li>
                  )}
                </For>
              </ul>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}
