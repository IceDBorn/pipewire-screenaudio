export type CommandStorageMap = {
	inMap?: [keyof LocalStorageTypes, string][];
	outMap?: [keyof LocalStorageTypes, string | null][];
};

export type BackgroundCommand<Command extends NativeMessaging.Commands> = {
	cmd: Command;
	args: NativeMessaging.Requests[Command];
	maps: CommandStorageMap;
	event?: string;
};
