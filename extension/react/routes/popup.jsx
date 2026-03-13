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

const isChromium = typeof browser === "undefined";

function mapNode(node) {
	return {
		mediaName: node.properties["media.name"],
		applicationName: node.properties["application.name"],
		serial: node.properties["object.serial"],
		checked: false,
	};
}

/**
 * @param {Object} param0
 * @param {object[]} param0.nodes
 * @param {boolean} param0.areNodesLoading
 */
function useNodeSelectionState({ nodes, areNodesLoading }) {
	const {
		data: storedNodeSelection,
		setData: setStoredNodeSelection,
		isLoading,
	} = useLocalStorage(SELECTED_NODES);

	if (isLoading || areNodesLoading) return { isLoading: true };

	let nodeSelection;

	if (!nodes) {
		nodeSelection = new Set(
			(storedNodeSelection ?? []).map((node) => node.serial),
		);
	} else {
		nodeSelection = new Set(
			(storedNodeSelection ?? [])
				.filter((selectedNode) =>
					nodes.some((node) => matchNode(node, selectedNode)),
				)
				.map((node) => node.serial),
		);
	}

	/**
	 * @type {function(int[]):void}
	 */
	const toggleNodes = (serials) => {
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
	const [versionMatches, setVersionMatches] = useState(null);
	const [connectorConnection, setConnectorConnection] = useState(false);
	const [extensionVersion, setExtensionVersion] = useState(null);
	const [nativeVersion, setNativeVersion] = useState(null);

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

/**
 * @param {Object} param0
 * @param {boolean} param0.enabled
 */
function useNodes({ enabled }) {
	const [nodes, setNodes] = useState(null);
	const [isInitialized, setIsInitialized] = useState(false);
	const [isErrored, setIsErrored] = useState(false);

	useEffect(() => {
		if (!enabled) return;

		let previousNodes = null;
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

	return { nodes, isErrored, isInitialized };
}

/**
 * @param {Object} param0
 * @param {boolean} param0.enabled
 */
function useCurrentMicId({ enabled }) {
	const [isRunning, setIsRunning] = useState(null);
	const {
		isLoading: isLocalStorageLoading,
		data: micId,
		setData: setMicId,
	} = useLocalStorage(MIC_ID);
	const [isInitialized, setIsInitialized] = useState(false);

	const shouldListen = enabled && !isLocalStorageLoading;

	function handleMicIdUpdated(id) {
		if (!id) return;
		setMicId(id.detail.micId);
		setIsRunning(true);
	}

	function handleMicIdRemoved() {
		setMicId(null);
		setIsRunning(false);
	}

	useEffect(() => {
		if (!shouldListen) return;

		let func = async () => {
			try {
				const res = await isPipewireScreenAudioRunning(micId);
				console.log({ res, micId });
				if (!res) setMicId(null);
				setIsRunning(res);

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

	return { isInitialized, micId, isRunning };
}

function useAllDesktopAudio() {
	const { isLoading, data, setData } = useLocalStorage(ALL_DESKTOP);

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

	const {
		isInitialized: isCurrentMicIdInitialized,
		isRunning,
		micId,
	} = useCurrentMicId({
		enabled: !isHealthcheckLoading && connectorConnection,
	});

	const { isAllDesktopAudioLoading, allDesktopAudio, setAllDesktopAudio } =
		useAllDesktopAudio();

	const {
		isLoading: isNodeSelectionLoading,
		nodeSelection,
		toggleNodes,
	} = useNodeSelectionState({
		nodes,
		areNodesLoading: !nodesSuccessfullyLoaded,
	});

	const debouncedSetSharingNodes = useDebouncedCallback(() => {
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
		!isHealthcheckLoading && versionMatches && !showConnectionError;

	function shareNodes(allDesktopAudio) {
		if (!isHealthy || !isRunning || allDesktopAudio) return;
		debouncedSetSharingNodes();
	}

	async function handleStartStop() {
		if (!isRunning) {
			startPipewireScreenAudio();
			if (allDesktopAudio) {
				setSharingNode([-1]);
			} else {
				setSharingNode(Array.from(nodeSelection));
			}
		} else {
			stopPipewireScreenAudio(micId);
		}
	}

	return (
		!isHealthcheckLoading && (
			<>
				<AppBar position="static" sx={{ maxWidth: 500 }}>
					<Toolbar>
						<Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
							Pipewire Screenaudio
						</Typography>
					</Toolbar>
				</AppBar>
				{(showConnectionError ||
					!versionMatches ||
					(isCurrentMicIdInitialized && isRunning)) && (
					<Alert
						severity={isRunning ? "info" : "error"}
						color={isRunning ? "info" : "error"}
						sx={{ maxWidth: 500 }}
					>
						{showConnectionError
							? "The native connector is missing or misconfigured"
							: !versionMatches
								? `Version mismatch! Extension: ${extensionVersion}, Native: ${nativeVersion}`
								: isRunning
									? `Running with ID: ${micId}`
									: console.error("unreachable")}
					</Alert>
				)}
				{/* Content */}
				{areNodesInitialized &&
					(areNodesErrored || !nodes.length ? (
						<Paper sx={{ minWidth: 500, minHeight: 80, borderRadius: 0 }}>
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
								{areNodesErrored
									? "Could not retrieve node list"
									: "No nodes available for sharing"}
							</Typography>
						</Paper>
					) : (
						<NodesTable
							nodes={nodes}
							nodeSelection={nodeSelection}
							toggleNodes={(serials) => {
								toggleNodes(serials);
								shareNodes(allDesktopAudio);
							}}
							hasError={!isHealthy}
							allDesktopAudio={allDesktopAudio}
						/>
					))}
				<Paper sx={{ maxWidth: 500, borderRadius: "0", padding: 1 }}>
					<Grid container justify="space-between">
						<span
							title={isChromium ? "Not supported on Chromium browsers" : ""}
						>
							<FormControlLabel
								control={
									<Switch
										onChange={() => {
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
								disabled={!isHealthy || isChromium || isAllDesktopAudioLoading}
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
