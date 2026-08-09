export function unreachable(message: string | undefined = undefined): never {
	if (message !== undefined)
		throw new Error(`Unreachable code executed: ${message}`);
	else throw new Error(`Unreachable code executed`);
}
