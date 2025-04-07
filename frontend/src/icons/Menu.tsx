import { ComponentProps } from "solid-js";

export function MenuIcon(props: ComponentProps<"svg">) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 20 20"
      {...props}
    >
      <path
        d="M4 5H16a1 1 0 010 2H4a1 1 0 010-2Zm0 4H16a1 1 0 010 2H4a1 1 0 010-2Zm0 4H16a1 1 0 010 2H4a1 1 0 010-2Z"
        fill="currentColor"
      />
    </svg>
  );
}
