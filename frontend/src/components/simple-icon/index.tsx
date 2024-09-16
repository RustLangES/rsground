import { ReactElement } from "react";
import { SimpleIcon } from "simple-icons"

interface SimpleIconProps {
	icon: SimpleIcon;
}

export default function SimpleIconSvg({icon}: SimpleIconProps): ReactElement {
	return <svg
		role="img"
		viewBox="0 0 24 24"
		xmlns="http://www.w3.org/2000/svg"
	>
		<title>{icon.title}</title>
		<path d={icon.path} />
	</svg>
}
