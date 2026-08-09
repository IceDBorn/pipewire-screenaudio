// TODO: Add nodes sorting
import React, { useCallback, useEffect, useState } from "react";

import Alert from "@mui/material/Alert";
import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import FormControlLabel from "@mui/material/FormControlLabel";
import Grid from "@mui/material/Grid";
import Paper from "@mui/material/Paper";
import Switch from "@mui/material/Switch";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import { useDebouncedCallback } from "use-debounce";

import {
	ERROR_VERSION_MISMATCH,
	EVENT_MIC_ID_UPDATED,
	EVENT_MIC_ID_REMOVED,
	healthCheck,
	getNodes,
	isChromium,
	isIncognito,
	isPipewireScreenAudioRunning,
	startPipewireScreenAudio,
	stopPipewireScreenAudio,
	setSharingNode,
} from "../lib/backend";

import {
	MIC_ID,
	ALL_DESKTOP,
	readLocalStorage,
	updateLocalStorage,
	SELECTED_NODES,
} from "../lib/local-storage";
import { useDidUpdateEffect, useLocalStorage } from "../lib/hooks";

import NodesTable from "../components/nodes-table";
import matchNode from "../lib/match-node";
import { unreachable } from "../lib/utils";

import type { PwNode, MicIdUpdatedEvent } from "../lib/types";
import type * as NativeMessaging from "../lib/nativeMessageTypes";

function mapNode(node: NativeMessaging.PwNode): PwNode {
	return {
		mediaName: node.properties["media.name"],
		applicationName: node.properties["application.name"],
		serial: node.properties["object.serial"],
	};
}

function useNodeSelectionState({
	nodes,
	areNodesLoading,
}:
	| { nodes: PwNode[]; areNodesLoading: false }
	| { nodes: any; areNodesLoading: true }):
	| { isLoading: true; nodeSelection: undefined; toggleNodes: undefined }
	| {
			isLoading: false;
			nodeSelection: Set<number>;
			toggleNodes: (serials: number[] | null) => void;
	  } {
	const {
		data: storedNodeSelection,
		setData: setStoredNodeSelection,
		isLoading,
	} = useLocalStorage(SELECTED_NODES);

	if (isLoading || areNodesLoading)
		return {
			isLoading: true,
			nodeSelection: undefined,
			toggleNodes: undefined,
		};

	const nodeSelection = new Set(
		(storedNodeSelection ?? [])
			.filter((selectedNode) =>
				nodes.some((node) => matchNode(node, selectedNode)),
			)
			.map((node) => node.serial),
	);

	const toggleNodes = (serials: number[] | null) => {
		let newNodeSelection;
		if (serials === null) {
			const turnOn = !nodes.every((node) => nodeSelection.has(node.serial));
			newNodeSelection = new Set(
				turnOn ? nodes.map((node) => node.serial) : [],
			);
		} else {
			newNodeSelection = new Set(nodeSelection);
			for (const serial of serials) {
				if (newNodeSelection.has(serial)) {
					newNodeSelection.delete(serial);
				} else {
					newNodeSelection.add(serial);
				}
			}
		}
		setStoredNodeSelection(
			nodes.filter((node) => newNodeSelection.has(node.serial)),
		);
	};

	return { isLoading: false, nodeSelection, toggleNodes };
}

function useHealthchecks() {
	const [isLoading, setIsLoading] = useState(true);
	const [versionMatches, setVersionMatches] = useState<boolean | null>(null);
	const [connectorConnection, setConnectorConnection] = useState(false);
	const [extensionVersion, setExtensionVersion] = useState<string | null>(null);
	const [nativeVersion, setNativeVersion] = useState<string | null>(null);

	useEffect(() => {
		(async () => {
			try {
				const health = await healthCheck();
				setVersionMatches(health);
				setConnectorConnection(true);
			} catch (err) {
				if (err.message === ERROR_VERSION_MISMATCH) {
					setNativeVersion(err.cause.nativeVersion);
					setExtensionVersion(err.cause.extensionVersion);
					setVersionMatches(false);
					setConnectorConnection(true);
				}
			}
			setIsLoading(false);
		})();
	}, []);

	return {
		isLoading,
		versionMatches,
		connectorConnection,
		extensionVersion,
		nativeVersion,
	};
}

function useNodes({ enabled }: { enabled: boolean }) {
	const [nodes, setNodes] = useState<PwNode[] | null>(null);
	const [isInitialized, setIsInitialized] = useState(false);
	const [isErrored, setIsErrored] = useState(false);

	useEffect(() => {
		if (!enabled) return;

		let previousNodes: string | null = null;
		const nodesReceive = async () => {
			try {
				const n = await getNodes();
				const currentNodesStr = JSON.stringify(n);
				if (currentNodesStr !== previousNodes) setNodes(n.map(mapNode));
				previousNodes = currentNodesStr;
				setIsErrored(false);
			} catch (err) {
				console.error("error receiving nodes: ", err);
				setNodes(null);
				setIsErrored(true);
			}
			if (!isInitialized) setIsInitialized(true);
		};
		nodesReceive();

		let nodesInterval = setInterval(nodesReceive, 2000);

		return () => {
			if (nodesInterval !== null) {
				clearInterval(nodesInterval);
			}
		};
	}, [enabled]);

	return !isInitialized
		? { nodes: null, isErrored: false, isInitialized }
		: isErrored
			? { nodes: null, isErrored, isInitialized }
			: nodes !== null
				? { nodes, isErrored, isInitialized }
				: unreachable();
}

function useCurrentMicId({ enabled }: { enabled: boolean }) {
	const {
		isLoading: isLocalStorageLoading,
		data: micId,
		setData: setMicId,
	} = useLocalStorage(MIC_ID);
	const [isInitialized, setIsInitialized] = useState(false);

	const shouldListen = enabled && !isLocalStorageLoading;

	useEffect(() => {
		if (!shouldListen) return;

		function handleMicIdUpdated(id: MicIdUpdatedEvent) {
			if (isLocalStorageLoading) unreachable();
			setMicId(id.detail.micId);
		}

		function handleMicIdRemoved() {
			if (isLocalStorageLoading) unreachable();
			setMicId(null);
		}

		let func = async () => {
			try {
				if (micId !== null) {
					const res = await isPipewireScreenAudioRunning(micId);
					console.log({ res, micId });
					if (!res) setMicId(null);
				}

				document.addEventListener(EVENT_MIC_ID_UPDATED, handleMicIdUpdated);
				document.addEventListener(EVENT_MIC_ID_REMOVED, handleMicIdRemoved);
			} catch (err) {
				console.error("unhandled error:", err);
			}
			setIsInitialized(true);
		};

		func();

		return () => {
			document.removeEventListener(EVENT_MIC_ID_UPDATED, handleMicIdUpdated);
			document.removeEventListener(EVENT_MIC_ID_REMOVED, handleMicIdRemoved);
		};
	}, [shouldListen]);

	if (!isInitialized)
		return { isInitialized, micId: undefined, isRunning: undefined };

	if (micId === undefined) unreachable("should be set if initialized");

	return { isInitialized, micId };
}

function useAllDesktopAudio():
	| {
			isAllDesktopAudioLoading: true;
			allDesktopAudio: undefined;
			setAllDesktopAudio: undefined;
	  }
	| {
			isAllDesktopAudioLoading: false;
			allDesktopAudio: boolean;
			setAllDesktopAudio: (value: boolean) => void;
	  } {
	const { isLoading, data, setData } = useLocalStorage(ALL_DESKTOP);

	if (isLoading)
		return {
			isAllDesktopAudioLoading: isLoading,
			allDesktopAudio: data,
			setAllDesktopAudio: setData,
		};

	return {
		isAllDesktopAudioLoading: isLoading,
		allDesktopAudio: !!data,
		setAllDesktopAudio: setData,
	};
}

export default function Popup() {
	const {
		isLoading: isHealthcheckLoading,
		connectorConnection,
		versionMatches,
		nativeVersion,
		extensionVersion,
	} = useHealthchecks();

	const {
		nodes,
		isErrored: areNodesErrored,
		isInitialized: areNodesInitialized,
	} = useNodes({
		enabled: !isHealthcheckLoading && connectorConnection,
	});
	const nodesSuccessfullyLoaded = areNodesInitialized && !areNodesErrored;

	const { isInitialized: isCurrentMicIdInitialized, micId } = useCurrentMicId({
		enabled: !isHealthcheckLoading && connectorConnection,
	});

	const isRunning = isCurrentMicIdInitialized && micId !== null;

	const { isAllDesktopAudioLoading, allDesktopAudio, setAllDesktopAudio } =
		useAllDesktopAudio();

	const {
		isLoading: isNodeSelectionLoading,
		nodeSelection,
		toggleNodes,
	} = useNodeSelectionState(
		nodesSuccessfullyLoaded
			? {
					nodes,
					areNodesLoading: false,
				}
			: {
					nodes: undefined,
					areNodesLoading: true,
				},
	);

	const nodeSelectionSuccessfullyLoaded =
		nodesSuccessfullyLoaded && !isNodeSelectionLoading;

	const shareNodes = useDebouncedCallback((allDesktopAudio: boolean) => {
		if (!isHealthy || !isRunning || allDesktopAudio) return;
		if (isNodeSelectionLoading) unreachable();
		setSharingNode(Array.from(nodeSelection));
	}, 1000);

	const debouncedShareAllDesktopAudio = useDebouncedCallback(() => {
		if (allDesktopAudio) {
			setSharingNode([-1]);
		} else {
			setSharingNode([]);
		}
	}, 1000);

	const showConnectionError = !connectorConnection || areNodesErrored;
	const isHealthy =
		!isHealthcheckLoading &&
		versionMatches &&
		nodeSelectionSuccessfullyLoaded &&
		!isAllDesktopAudioLoading;

	async function handleStartStop() {
		if (!isRunning) {
			startPipewireScreenAudio();
			if (allDesktopAudio) {
				setSharingNode([-1]);
			} else {
				if (!nodeSelectionSuccessfullyLoaded)
					unreachable(
						"start/stop button is clickable only after node selection is loaded",
					);
				setSharingNode(Array.from(nodeSelection));
			}
		} else {
			stopPipewireScreenAudio(micId);
		}
	}

	return (
		!isHealthcheckLoading && (
			<>
				<AppBar position="static">
					<Toolbar>
						<Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
							Pipewire Screenaudio
						</Typography>
					</Toolbar>
				</AppBar>
				{(showConnectionError || !versionMatches || isRunning) && (
					<Alert
						severity={isRunning ? "info" : "error"}
						color={isRunning ? "info" : "error"}
					>
						{showConnectionError
							? "The native connector is missing or misconfigured"
							: !versionMatches
								? `Version mismatch! Extension: ${extensionVersion}, Native: ${nativeVersion}`
								: isRunning
									? `Running with ID: ${micId}`
									: unreachable()}
					</Alert>
				)}
				{/* Content */}
				{(nodeSelectionSuccessfullyLoaded || showConnectionError) &&
					((nodeSelectionSuccessfullyLoaded && !nodes.length) ||
					showConnectionError ? (
						<Paper sx={{ minHeight: 80, borderRadius: 0 }}>
							<div></div>
							<Typography
								variant="h6"
								component="div"
								sx={{
									flexGrow: 1,
									marginLeft: 13,
									paddingTop: 5,
									paddingBottom: 5,
								}}
							>
								{showConnectionError
									? "Could not retrieve node list"
									: "No nodes available for sharing"}
							</Typography>
						</Paper>
					) : nodeSelectionSuccessfullyLoaded ? (
						<NodesTable
							nodes={nodes}
							nodeSelection={nodeSelection}
							toggleNodes={(serials) => {
								if (!isHealthy)
									unreachable(
										"inner interactions are disabled while !isHealthy, so this wont be called",
									);
								toggleNodes(serials);
								shareNodes(allDesktopAudio);
							}}
							disableInteraction={!isHealthy || allDesktopAudio}
						/>
					) : (
						unreachable()
					))}
				<Paper sx={{ borderRadius: "0", padding: 1 }}>
					<Grid container>
						<span
							title={isChromium() ? "Not supported on Chromium browsers" : ""}
						>
							<FormControlLabel
								control={
									<Switch
										onChange={() => {
											if (isAllDesktopAudioLoading)
												unreachable("switch is disabled while loading");
											const currentAllDesktopAudio = !allDesktopAudio;
											setAllDesktopAudio(currentAllDesktopAudio);

											if (currentAllDesktopAudio) {
												debouncedShareAllDesktopAudio();
											} else {
												shareNodes(currentAllDesktopAudio);
											}
										}}
									/>
								}
								sx={{ marginLeft: 0 }}
								label="All Desktop Audio"
								checked={allDesktopAudio}
								disabled={
									!isHealthy ||
									isAllDesktopAudioLoading ||
									isChromium() ||
									isIncognito()
								}
							/>
						</span>
						<Button
							sx={{
								minWidth: 75,
								marginLeft: "auto",
							}}
							variant="contained"
							color={isRunning ? "error" : "success"}
							onClick={handleStartStop}
							disabled={!isHealthy || !isCurrentMicIdInitialized}
						>
							{isRunning ? "Stop" : "Start"}
						</Button>
					</Grid>
				</Paper>
			</>
		)
	);
}
