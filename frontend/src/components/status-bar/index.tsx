import { ReactElement } from "react";
import { siGithub } from "simple-icons";
import { CircleAlert, Cuboid, TriangleAlert } from "lucide-react";

import SimpleIconSvg from "../simple-icon";

import "./styles.css";

interface StatusBarLoadable {
	text?: string;
	loading: boolean;
}

interface CodeInsights {
	errors: number;
	warnings: number;
	onChange?(errors: number, warnings: number): void;
}

interface CursorInsights {
	line: number;
	char: number;
	onChange?(line: number, char: number): void;
}

interface StatusBarProps {
	github: StatusBarLoadable;
	insights: CodeInsights;
	container: StatusBarLoadable;
	cursor?: CursorInsights;
}

export default function StatusBar({ github, insights, container, cursor }: StatusBarProps): ReactElement {

	console.info(github, insights, container, cursor); //  Build fails if not used

	return <div className="status-bar">
		<div className="status-bar-left">
			<div className="status-generic">
				<p>Not Synced</p>
				<SimpleIconSvg icon={siGithub} />
			</div>
			<div className="status-insights">
				<div>
					<p>0</p>
					<CircleAlert />
				</div>
				<div>
					<p>0</p>
					<TriangleAlert />
				</div>
			</div>
			<div className="status-generic">
				<p>Not connected</p>
				<Cuboid />
			</div>
		</div>
		<div className="status-bar-right">
			<div className="status-line-count">
				<p>0:0</p>
			</div>
			<div className="status-indentation">
				<p>4 spaces</p>
			</div>
		</div>
	</div>;
}
