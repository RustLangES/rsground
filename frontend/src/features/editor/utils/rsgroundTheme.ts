import { HighlightStyle } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

import styles from "./rsgroundTheme.module.sass";

export const rsgroundTheme = HighlightStyle.define([
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
