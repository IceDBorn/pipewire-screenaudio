/** @import { PwNode } from "./types" */

/**
 * @param {PwNode} a
 * @param {PwNode} b
 */
export default function matchNode(a, b) {
	return (
		a.serial === b.serial &&
		a.mediaName === b.mediaName &&
		a.applicationName === b.applicationName
	);
}
