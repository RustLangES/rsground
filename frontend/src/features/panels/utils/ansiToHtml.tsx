import styles from "./ansiToHtml.module.sass";

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

export function ansiToHtml(text: string): HTMLElement {
  let node = <pre /> as HTMLElement;

  let remaining = text;
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

      // Add node only if there are non-whitespace characters
      if (accumulatedContent.trimEnd().length) {
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

  return node;
}

function addAnsiStyles(node: HTMLElement, ansiCode: string) {
  const is8BitForeground = ansiCode.startsWith("3" + ANSI_8_BIT_COLOR);
  const is8BitBackground = !is8BitForeground &&
    ansiCode.startsWith("4" + ANSI_8_BIT_COLOR);

  if (is8BitForeground || is8BitBackground) {
    const parsedAnsiCode = parseInt(
      ansiCode.substring(1 + ANSI_8_BIT_COLOR.length),
      10,
    );

    let code: number;

    if (parsedAnsiCode <= 7) {
      code = parsedAnsiCode + (is8BitBackground ? 40 : 30);
    } else if (parsedAnsiCode <= 15) {
      // Normalize to 0, above 8 is bright color
      code = parsedAnsiCode - 8 + (is8BitBackground ? 100 : 90);
    }

    const klass = ANSI_8_BIT_COLORS[code];
    node.classList.add(klass);
  } else if (ansiCode.startsWith(ANSI_24_BIT_COLOR)) {
    // TODO: Support for 24bit ansi colors, just parse rgb
    console.warn("TODO");
  } else {
    const codes = ansiCode.split(";");

    for (let code of codes) {
      node.classList.add(ANSI_8_BIT_COLORS[parseInt(code)]);
    }
  }
}
