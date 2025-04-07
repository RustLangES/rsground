import { createSignal } from "solid-js";
import { CodeMirror } from "@solid-codemirror/codemirror";
import { basicSetup } from "codemirror";
import { rust, rustLanguage } from "@codemirror/lang-rust";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { autocompletion, CompletionContext } from "@codemirror/autocomplete";
import { tags as t } from "@lezer/highlight";

import styles from "./CodePanel.module.sass";

const rsgroundTheme = HighlightStyle.define([
  { tag: t.keyword, class: styles.keyword },
  {
    tag: [
      t.function(t.name),
      t.function(t.propertyName),
      t.labelName,
      t.macroName,
    ],
    class: styles.fn,
  },
  { tag: [t.typeName, t.namespace], class: styles.struct },
  { tag: t.string, class: styles.string },
  { tag: t.meta, class: styles.attribute },
  { tag: t.special(t.variableName), class: styles.lifetime },
]);

const keywords = [
  "pub",
  "fn",
  "struct",
  "let",
];

const localCompletions = [
  ...(keywords.map((keyword) => ({
    label: keyword,
    type: "keyword",
  }))),
];

const CODE_EXAMPLE = `
pub struct Something {
  prop: String
}

#[tokio::main]
fn main<'a>() {
  let a = String::new();
  println!(r#"{a}"#);
}`;

export function CodePanel() {
  const [code, setCode] = createSignal(CODE_EXAMPLE);

  return (
    <CodeMirror
      class={styles.container}
      value={code()}
      extensions={[
        basicSetup,
        syntaxHighlighting(rsgroundTheme),
        autocompletion({
          interactionDelay: 0,
          activateOnTypingDelay: 0,
          closeOnBlur: false,
          tooltipClass: () => styles.completion_tooltip,
          optionClass: (completion) => {
            const typeClass = completion.type
              ? " " + styles["completion_t_" + completion.type]
              : "";

            return styles.completion_option + typeClass;
          },
        }),
        rust(),
        rustLanguage.data.of({
          "autocomplete": async (context: CompletionContext) => {
            let word = context.matchBefore(/\w*/);
            if (word.from == word.to && !context.explicit) {
              return null;
            }

            return {
              from: word.from,
              options: localCompletions,
            };
          },
        }),
      ]}
    />
  );
}
