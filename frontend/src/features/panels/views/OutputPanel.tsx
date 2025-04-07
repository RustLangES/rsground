import { PlayIcon } from "@icons/Play";
import { SkullIcon } from "@icons/Skull";

import { ansiToHtml } from "../utils";

import styles from "./OutputPanel.module.sass";

const TEST =
  `\x1b[1m\x1b[32m   Compiling\x1b[0m nix-compiler v0.1.0 (/home/apika/dev/rust/nix-compiler)
\x1b[0m\x1b[1m\x1b[38;5;9merror[E0433]\x1b[0m\x1b[0m\x1b[1m: failed to resolve: could not find \`NixAttrSet\` in the crate root\x1b[0m
\x1b[0m   \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m--> \x1b[0m\x1b[0msrc/builtins/impl.rs:953:1\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m953\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m/\x1b[0m\x1b[0m \x1b[0m\x1b[0mgen_builtins! {\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m954\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m    currentSystem = NixValue::String("x86_64-linux".to_owned());\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m955\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m    false = NixValue::Bool(false);\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m956\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m    nixVersion = NixValue::String("2.24.9".to_owned());\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m957\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m    null = NixValue::Null;\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m958\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m    true = NixValue::Bool(true);\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;12m959\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m}\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9m|_^\x1b[0m\x1b[0m \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;9mcould not find \`NixAttrSet\` in the crate root\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m= \x1b[0m\x1b[0m\x1b[1mnote\x1b[0m\x1b[0m: this error originates in the macro \`gen_builtins\` (in Nightly builds, run with -Z macro-backtrace for more info)\x1b[0m
\x1b[0m\x1b[1m\x1b[38;5;14mhelp\x1b[0m\x1b[0m: consider importing this enum\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m
\x1b[0m    \x1b[1m\x1b[38;5;12m|\x1b[0m\x1b[0m   \x1b[0m\x1b[0m\x1b[38;5;10m+ use crate::value::NixAttrSet;\x1b[0m
\x1b[0m    \x1b[0m\x1b[0m\x1b[1m\x1b[38;5;12m|\x1b[0m

\x1b[0m\x1b[1mSome errors have detailed explanations: E0063, E0277, E0308, E0369, E0433, E0599.\x1b[0m
\x1b[0m\x1b[1mFor more information about an error, try \`rustc --explain E0063\`.\x1b[0m
\x1b[1m\x1b[31merror\x1b[0m\x1b[1m:\x1b[0m could not compile \`nix-compiler\` (bin "nix-compiler") due to 47 previous errors
`;

export function OutputPanel() {
  const node = ansiToHtml(TEST);

  return (
    <div class={styles.container}>
      <ul class={styles.actions} aria-label="Output actions">
        <li
          class={styles.action_play}
          aria-role="button"
          aria-label="Run code"
          title="Run code"
        >
          <PlayIcon />
        </li>

        <li
          class={styles.action_kill}
          aria-role="button"
          aria-label="Kill program"
          title="Kill program"
        >
          <SkullIcon />
        </li>
      </ul>

      <div class={styles.output}>
        {node}
      </div>
    </div>
  );
}
