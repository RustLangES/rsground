import { ComponentProps } from "solid-js";

export function Spinner(props: ComponentProps<"svg">) {
  return (
    <svg
      viewBox="-2 -2 42 42"
      xmlns="http://www.w3.org/2000/svg"
      fill="currentColor"
      data-test-name="oval"
      role="img"
      stroke="currentColor"
      {...props}
      width={props.width || 20}
      height={props.height || 20}
    >
      <g fill="none" fill-rule="evenodd">
        <g transform="translate(1 1)" stroke-width={props["stroke-width"] ?? 7}>
          <circle stroke-opacity=".5" cx="18" cy="18" r="18" />
          <path d="M36 18c0-9.94-8.06-18-18-18">
            <animateTransform
              attributeName="transform"
              type="rotate"
              from="0 18 18"
              to="360 18 18"
              dur="1s"
              repeatCount="indefinite"
            />
          </path>
        </g>
      </g>
    </svg>
  );
}
