import styles from "./OutputPanel.module.sass";

const TEST = `\x1b[1m\x1b[32m   Compiling\x1b[0m nix-compiler v0.1.0 (/home/apika/dev/rust/nix-compiler)
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

const ANSI_8_BIT_COLOR = "8;5;";
const ANSI_24_BIT_COLOR = "8;2;";

/* https://en.wikipedia.org/wiki/ANSI_escape_code#Colors */
const ANSI_8_BIT_COLORS = {
  [0]: styles.mods_reset,
  [1]: styles.mods_bold,
  [2]: styles.mods_dim,
  [3]: styles.mods_italic,
  [4]: styles.mods_underline,
  [30]: styles.fore_black,
  [40]: styles.back_black,
  [31]: styles.fore_red,
  [41]: styles.back_red,
  [32]: styles.fore_green,
  [42]: styles.back_green,
  [33]: styles.fore_yellow,
  [43]: styles.back_yellow,
  [34]: styles.fore_blue,
  [44]: styles.back_blue,
  [35]: styles.fore_magenta,
  [45]: styles.back_magenta,
  [36]: styles.fore_cyan,
  [46]: styles.back_cyan,
  [37]: styles.fore_white,
  [47]: styles.back_white,
  [90]: styles.fore_bright_black,
  [100]: styles.back_bright_black,
  [91]: styles.fore_bright_red,
  [101]: styles.back_bright_red,
  [92]: styles.fore_bright_green,
  [102]: styles.back_bright_green,
  [93]: styles.fore_bright_yellow,
  [103]: styles.back_bright_yellow,
  [94]: styles.fore_bright_blue,
  [104]: styles.back_bright_blue,
  [95]: styles.fore_bright_magenta,
  [105]: styles.back_bright_magenta,
  [96]: styles.fore_bright_cyan,
  [106]: styles.back_bright_cyan,
  [97]: styles.fore_bright_white,
  [107]: styles.back_bright_white,
};

function addAnsiStyles(node: HTMLElement, ansiCode: string) {
  if (
    ansiCode.startsWith("3" + ANSI_8_BIT_COLOR) ||
    ansiCode.startsWith("4" + ANSI_8_BIT_COLOR)
  ) {
    const isBg = ansiCode.startsWith("4");
    const parsedAnsiCode = parseInt(
      ansiCode.substring(1 + ANSI_8_BIT_COLOR.length),
      10,
    );

    if (parsedAnsiCode <= 7) {
      node.classList.add(
        ANSI_8_BIT_COLORS[parsedAnsiCode + (isBg ? 40 : 30)],
      );
    } else if (parsedAnsiCode <= 15) {
      node.classList.add(
        ANSI_8_BIT_COLORS[parsedAnsiCode - 8 + (isBg ? 100 : 90)],
      );
    } else {
    }
  } else if (ansiCode.startsWith(ANSI_24_BIT_COLOR)) {
    console.warn("TODO");
  } else {
    const codes = ansiCode.split(";");

    for (let code of codes) {
      node.classList.add(ANSI_8_BIT_COLORS[parseInt(code)]);
    }
  }
}

export function OutputPanel() {
  let node = <pre /> as HTMLElement;

  let remaining = TEST;
  let lastIndex = remaining.indexOf("\x1b");
  let lastNode = node;
  let lastCode = "";
  // For <span> reduction, needs to know all the content
  // from the last <span>
  let accumulatedContent = "";

  while (lastIndex !== -1) {
    // Append the raw text until the escape code
    const content = remaining.substring(0, lastIndex);
    lastNode.append(content);

    // Collect the ansi code
    const mIndex = remaining.indexOf("m", lastIndex);
    const ansiCode = remaining.substring(lastIndex + 2, mIndex);

    // Append the span with code
    accumulatedContent += content;

    if (ansiCode != lastCode) {
      const newNode = (accumulatedContent.length
        ? <span data-ansi={ansiCode} />
        : lastNode) as HTMLElement;

      addAnsiStyles(newNode, ansiCode);

      if (accumulatedContent.length) {
        // Reduce children depth
        if (newNode.dataset["ansi"] == "0") {
          node.append(newNode);
        } else {
          lastNode.append(newNode);
        }
        lastNode = newNode;
        accumulatedContent = "";
      }

      lastCode = ansiCode;
    }

    // Cut the string for the next iteration
    remaining = remaining.substring(mIndex + 1);
    lastIndex = remaining.indexOf("\x1b");
  }

  // Append remaining raw text
  lastNode.append(remaining);

  return (
    <div class={styles.container}>
      <ul class={styles.actions} aria-label="Output actions">
        <li class={styles.action_play} aria-role="button" aria-label="Run code" title="Run code">
          <svg
            aria-label="Play icon"
            fill="currentColor"
            stroke-width="0"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 384 512"
            style="overflow: visible; color: currentcolor;"
            height="1em"
            width="1em"
          >
            <path d="M73 39c-14.8-9.1-33.4-9.4-48.5-.9S0 62.6 0 80v352c0 17.4 9.4 33.4 24.5 41.9S58.2 482 73 473l288-176c14.3-8.7 23-24.2 23-41s-8.7-32.2-23-41L73 39z">
            </path>
          </svg>
        </li>

        <li class={styles.action_kill} aria-role="button" aria-label="Kill program" title="Kill program">
          <svg
            aria-label="Skull icon"
            fill="currentColor"
            stroke-width="0"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 512 512"
            style="overflow: visible; color: currentcolor;"
            height="1em"
            width="1em"
          >
            <path d="M416 398.9c58.5-41.1 96-104.1 96-174.9C512 100.3 397.4 0 256 0S0 100.3 0 224c0 70.7 37.5 133.8 96 174.9V464c0 26.5 21.5 48 48 48h48v-48c0-8.8 7.2-16 16-16s16 7.2 16 16v48h64v-48c0-8.8 7.2-16 16-16s16 7.2 16 16v48h48c26.5 0 48-21.5 48-48v-65.1zM96 256a64 64 0 1 1 128 0 64 64 0 1 1-128 0zm256-64a64 64 0 1 1 0 128 64 64 0 1 1 0-128z">
            </path>
          </svg>
        </li>
      </ul>

      <div class={styles.output}>
        {node}
      </div>
    </div>
  );
}
