export type Commands =
	| "GetNodes"
	| "IsPipewireScreenAudioRunning"
	| "GetVersion"
	| "StartPipewireScreenAudio"
	| "StopPipewireScreenAudio"
	| "SetSharingNode";

export type Requests = {
	GetNodes: undefined;
	IsPipewireScreenAudioRunning: { micId: number };
	GetVersion: undefined;
	StartPipewireScreenAudio: undefined;
	StopPipewireScreenAudio: { micId: number };
	SetSharingNode: { nodes: number[] };
};

export type PwNode = {
	id: number;
	properties: {
		["application.name"]: string | null;
		["media.name"]: string;
		["object.serial"]: number;
	};
};

export type Responses = {
	GetNodes: PwNode[];
	IsPipewireScreenAudioRunning: { isRunning: boolean };
	GetVersion: { version: string };
	StartPipewireScreenAudio: { micId: number };
	StopPipewireScreenAudio: undefined;
	SetSharingNode: undefined;
};
