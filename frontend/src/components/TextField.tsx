import { JSX } from "solid-js/jsx-runtime";
import { ComponentProps, splitProps } from "solid-js";

import styles from "./TextField.module.sass";

export type TextFieldProps = {
  /**
   * You can add styled interactivity by setting `beforeInteract` to true.
   * It is styled by default but you can override it with your own classes.
   */
  beforeIcon?: JSX.Element;
  beforeInteract?: boolean;
  /**
   * You can add styled interactivity by setting `afterInteract` to true.
   * It is styled by default but you can override it with your own classes.
   */
  afterIcon?: JSX.Element;
  afterInteract?: boolean;
};

export function TextField(props: TextFieldProps & ComponentProps<"input">) {
  const [fieldProps, restProps] = splitProps(props, [
    "afterIcon",
    "afterInteract",
    "beforeIcon",
    "beforeInteract",
    "class",
    "classList",
  ]);

  const hasBefore = !!fieldProps.beforeIcon;
  const hasAfter = !!fieldProps.afterIcon;
  const hasIcons = hasBefore || hasAfter;

  if (hasIcons) {
    return (
      <div
        classList={{
          [styles.group]: true,
          [styles.has_before]: hasBefore,
          [styles.has_before_interact]: hasBefore && fieldProps.beforeInteract,
          [styles.has_after]: hasAfter,
          [styles.has_after_interact]: hasAfter && fieldProps.afterInteract,
          [fieldProps.class]: true,
          ...(fieldProps.classList || {}),
        }}
      >
        {fieldProps.beforeIcon}
        <input {...restProps} />
        {fieldProps.afterIcon}
      </div>
    );
  }

  return (
    <input
      classList={{
        [styles.single]: true,
        [fieldProps.class]: true,
        ...(fieldProps.classList || {}),
      }}
      {...restProps}
    />
  );
}
