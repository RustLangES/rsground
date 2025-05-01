import { untrack } from "solid-js";
import SWAL, { SweetAlertOptions } from "sweetalert2";

import { ThemeMode, themeMode } from "@features/theme";

import styles from "./toast.module.sass";

const baseToast = SWAL.mixin({
  position: "bottom-right",
  toast: true,
  showConfirmButton: false,
  timer: 5_000,
  timerProgressBar: true,
  didOpen(toast) {
    toast.onmouseenter = SWAL.stopTimer;
    toast.onmouseleave = SWAL.resumeTimer;
  },
  showClass: {
    popup: styles.animation_show,
  },
  hideClass: {
    popup: styles.animation_hide,
  },
  customClass: {
    container: styles.container,
    popup: styles.popup,
  },
});

export function showToast<T = any>(
  kind: "debug" | "info" | "success" | "warn" | "error",
  options: SweetAlertOptions,
): ReturnType<typeof baseToast.fire<T>> {
  return baseToast.fire<T>({
    theme: untrack(themeMode) === ThemeMode.System
      ? "auto"
      : untrack(themeMode) === ThemeMode.Dark
      ? "dark"
      : "light",
    icon: options.icon ||
      (kind === "debug"
        ? "info"
        : kind === "success"
        ? "success"
        : kind === "warn"
        ? "warning"
        : kind === "error"
        ? "error"
        : undefined),
    iconColor: "var(--icon-color)",
    customClass: {
      container: styles["container-" + kind],
      popup: styles.popup,
      ...options.customClass,
    },
    ...options,
  });
}
