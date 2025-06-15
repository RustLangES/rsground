import Dialog from "@corvu/dialog";
import { createEffect, createSignal, For, Show } from "solid-js";

import { SelectField } from "@components/SelectField";
import { Switchbox } from "@components/Switchbox";
import { TextField } from "@components/TextField";
import { AccessLevel, ClientMessageKind } from "@features/ws/types";
import { sendMessage } from "@features/ws/services";
import { LockIcon } from "@icons/Lock";
import { BrandsGithubIcon } from "@icons/BrandsGithub";
import { showToast } from "@services/toast";

import {
  isColabOpen,
  isProjectOwner,
  projectInfo,
  setIsColabOpen,
  setProjectInfo,
} from "../stores";
import { forkProject } from "../services";

import styles from "./Colab.module.sass";
import { AddIcon } from "@icons/Add";
import { SolidUserIcon } from "@icons/SolidUser";

export function Colab() {
  const [addMenu, setAddMenu] = createSignal(false);

  return (
    <Dialog open={isColabOpen()} onOpenChange={setIsColabOpen}>
      <Dialog.Portal>
        <Dialog.Overlay class={styles.overlay} />
        <Dialog.Content class={styles.content}>
          <h2 class={styles.title}>Live Collaboration</h2>
          <ColabButtons />

          <div class={styles.container}>
            <Show when={isProjectOwner()}>
              <div>
                <h3 class={styles.subtitle}>Room settings</h3>
                <ColabPublicPassword />
              </div>
            </Show>

            <div>
              <Show when={!addMenu()}>
                <div class={styles.title_button_container}>
                  <h3 class={styles.subtitle}>Members</h3>
                  <Show when={isProjectOwner()}>
                    <button onClick={() => setAddMenu(true)}>
                      <AddIcon />
                    </button>
                  </Show>
                </div>

                <ul class={styles.user_list}>
                  <For each={Object.entries(projectInfo.users)}>
                    {([user_id, [username, access]]) => (
                      <li class={styles.member}>
                        <span class={styles.member_name}>{username}</span>

                        <SelectField
                          value={access}
                          disabled={!isProjectOwner()}
                          options={[AccessLevel.Editor, AccessLevel.ReadOnly]}
                          onValueChange={(access) => {
                            sendMessage(ClientMessageKind.PermitAccess, {
                              user_id,
                              access,
                            });
                          }}
                        />
                      </li>
                    )}
                  </For>
                </ul>
              </Show>

              <Show when={addMenu()}>
                <div class={styles.title_button_container}>
                  <h3 class={styles.subtitle}>Add</h3>
                  <Show when={isProjectOwner()}>
                    <button onClick={() => setAddMenu(false)}>
                      <SolidUserIcon />
                    </button>
                  </Show>
                </div>

                <TextField
                  beforeIcon={<BrandsGithubIcon />}
                  placeholder="Username"
                />
                <ul class={styles.user_list}>
                  <For each={Object.entries(projectInfo.requests)}>
                    {([user, name]) => (
                      <li class={styles.member}>
                        <span class={styles.member_name}>{name}</span>

                        <ul class={styles.button_group}>
                          <button
                            class={styles.success}
                            onClick={() => {
                              setProjectInfo("users", user, [
                                name,
                                AccessLevel.ReadOnly,
                              ]);
                              sendMessage(ClientMessageKind.PermitAccess, {
                                user_id: user,
                                access: AccessLevel.ReadOnly,
                              });
                            }}
                          >
                            Allow
                          </button>
                          <button
                            class={styles.error}
                            onClick={() => {
                              setProjectInfo("requests", user, undefined);
                            }}
                          >
                            Kick
                          </button>
                        </ul>
                      </li>
                    )}
                  </For>
                </ul>
              </Show>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}

function ColabPublicPassword() {
  const [password, setPassword] = createSignal("");

  let debounce: NodeJS.Timeout;
  let first_time = true;
  createEffect(() => {
    let pass = password();

    if (first_time && ((first_time = false), true)) return;

    if (debounce) clearTimeout(debounce);

    debounce = setTimeout(() => {
      sendMessage(ClientMessageKind.Config, { password: pass });
    }, 500);
  });

  return (
    <>
      <label class={styles.checkbox_input}>
        Public room
        <Switchbox
          checked={projectInfo.is_public}
          onChange={(ev) => {
            setProjectInfo("is_public", ev.currentTarget.checked);
            sendMessage(ClientMessageKind.Config, {
              is_public: ev.currentTarget.checked,
            });
          }}
        />
      </label>

      <Show when={projectInfo.is_public}>
        <TextField
          value={projectInfo.password ?? ""}
          onInput={(ev) => setPassword(ev.currentTarget.value)}
          beforeIcon={<LockIcon />}
          placeholder="Leave empty for no password"
        />
        <span class={styles.password_hint}>Password is visible</span>
      </Show>
    </>
  );
}

function ColabButtons() {
  const copyPath = (suffix = "") => {
    navigator.clipboard.writeText(
      `${location.protocol}//${location.host}/${projectInfo.id}${suffix}`,
    );
    showToast("success", { text: "Link copied" });
  };

  return (
    <div class={styles.buttons_container}>
      <button onClick={() => copyPath()}>Copy colab link</button>
      <button onClick={() => copyPath("/fork")}>Copy fork link</button>
      <button onClick={() => forkProject(projectInfo.id)}>Fork</button>
    </div>
  );
}
