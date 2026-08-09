import { ALL_DESKTOP, MIC_ID, SELECTED_NODES } from "./local-storage";

export type UseLocalStorageResult<T> =
	| {
			isLoading: false;
			data: T | null;
			setData: (value: T) => void;
	  }
	| {
			isLoading: true;
			data: undefined;
			setData: undefined;
	  };

export type PwNode = {
	serial: number;
	mediaName: string;
	applicationName: string | null;
};

export type LocalStorageTypes = {
	[SELECTED_NODES]: PwNode[];
	[MIC_ID]: number | null;
	[ALL_DESKTOP]: boolean;
};

export type MicIdUpdatedEvent = CustomEvent<{ micId: number }>;
