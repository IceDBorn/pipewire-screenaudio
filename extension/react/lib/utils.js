/**
 * @param {string | undefined} message
 * @returns {never}
 */
export function unreachable(message = undefined) {
	if (message !== undefined)
		throw new Error(`Unreachable code executed: ${message}`);
	else throw new Error(`Unreachable code executed`);
}
