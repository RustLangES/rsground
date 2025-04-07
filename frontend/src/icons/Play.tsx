import { ComponentProps } from "solid-js";

export function PlayIcon(props: ComponentProps<"svg">) {
  return (
    <svg
      aria-label="Play icon"
      fill="currentColor"
      stroke-width="0"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 384 512"
      style="overflow: visible; color: currentcolor;"
      height="1em"
      width="1em"
      {...props}
    >
      <path d="M73 39c-14.8-9.1-33.4-9.4-48.5-.9S0 62.6 0 80v352c0 17.4 9.4 33.4 24.5 41.9S58.2 482 73 473l288-176c14.3-8.7 23-24.2 23-41s-8.7-32.2-23-41L73 39z">
      </path>
    </svg>
  );
}
