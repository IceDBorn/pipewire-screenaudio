import type { PwNode } from "./types";

export default function matchNode(a: PwNode, b: PwNode) {
	return (
		a.serial === b.serial &&
		a.mediaName === b.mediaName &&
		a.applicationName === b.applicationName
	);
}
