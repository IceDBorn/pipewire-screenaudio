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

function mapNode(node) {
  return {
    mediaName: node.properties["media.name"],
    applicationName: node.properties["application.name"],
    serial: node.properties["object.serial"],
    checked: false,
  };
}

function useNodeSelectionState(nodes) {
  const [nodeSelection, setNodeSelection] = useState(new Set());

  useEffect(() => {
    if (nodes === null) return;
    // only keep nodes that actually exist
    const selectedNodes = readLocalStorage(SELECTED_NODES) ?? [];
    setNodeSelection(
      new Set(
        selectedNodes
          .filter((selectedNode) =>
            nodes.some((node) => matchNode(node, selectedNode)),
          )
          .map((node) => node.serial),
      ),
    );
  }, [nodes]);

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
    updateLocalStorage(
      SELECTED_NODES,
      nodes.filter((node) => newNodeSelection.has(node.serial)),
    );
    setNodeSelection(newNodeSelection);
  };

  return { nodeSelection, toggleNodes };
}

export default function Popup() {
  const [versionMatches, setVersionMatches] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [connectorConnection, setConnectorConnection] = useState(false);
  const [allDesktopAudio, setAllDesktopAudio] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [nativeVersion, setNativeVersion] = useState("");
  const [extensionVersion, setExtensionVersion] = useState("");
  const [nodes, setNodes] = useState(null);
  const [micId, setMicId] = useLocalStorage(MIC_ID);
  const { nodeSelection, toggleNodes } = useNodeSelectionState(nodes);
  const isHealthy = versionMatches && connectorConnection;

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

  useEffect(() => {
    let nodesInterval = null;
    let micIdUpdatedEventListener = null;
    let micIdRemovedEventListener = null;

    let func = async () => {
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

        setIsInitialized(true);
        return;
      }

      let previousNodes = null;
      const nodesReceive = () =>
        getNodes().then(
          (n) => {
            const currentNodesStr = JSON.stringify(n);
            if (currentNodesStr === previousNodes) return;
            previousNodes = currentNodesStr;
            setNodes(n.map(mapNode));
          },
          (err) => {
            console.error("unhandled error:", err);
            setIsInitialized(true);
          },
        );
      nodesReceive();
      nodesInterval = setInterval(nodesReceive, 2000);

      if (micId) {
        try {
          const res = await isPipewireScreenAudioRunning(micId);
          console.log({ res, micId });
          setIsRunning(res);
        } catch {
          console.error("unhandled error:", err);
          setIsInitialized(true);
          return;
        }
      }

      setAllDesktopAudio(readLocalStorage(ALL_DESKTOP));

      micIdUpdatedEventListener = document.addEventListener(
        EVENT_MIC_ID_UPDATED,
        handleMicIdUpdated,
      );
      micIdRemovedEventListener = document.addEventListener(
        EVENT_MIC_ID_REMOVED,
        handleMicIdRemoved,
      );

      setIsInitialized(true);
    };

    func();

    return () => {
      if (nodesInterval !== null) {
        clearInterval(nodesInterval);
      }
      if (micIdUpdatedEventListener !== null) {
        document.removeEventListener(micIdUpdatedEventListener);
      }
      if (micIdRemovedEventListener !== null) {
        document.removeEventListener(micIdRemovedEventListener);
      }
    };
  }, []);

  useDidUpdateEffect(() => {
    if (isRunning) {
      const id = readLocalStorage(MIC_ID);
      console.log({ isRunning, id });
      setMicId(id);
    } else {
      console.log({ isRunning, id: null });
      setMicId(null);
    }
  }, [isRunning]);

  function handleMicIdUpdated(id) {
    if (!id) return;
    setIsRunning(true);
  }

  function handleMicIdRemoved() {
    setIsRunning(false);
  }

  function shareNodes(allDesktopAudio) {
    if (!isRunning || allDesktopAudio) return;
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
    isInitialized && (
      <>
        <AppBar position="static" sx={{ maxWidth: 500 }}>
          <Toolbar>
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Pipewire Screenaudio
            </Typography>
          </Toolbar>
        </AppBar>
        {(!versionMatches || !connectorConnection || isRunning) && (
          <Alert
            severity={isRunning ? "info" : "error"}
            color={isRunning ? "info" : "error"}
            sx={{ maxWidth: 500 }}
          >
            {!connectorConnection
              ? "The native connector is missing or misconfigured"
              : !versionMatches
                ? `Version mismatch! Extension: ${extensionVersion}, Native: ${nativeVersion}`
                : isRunning
                  ? `Running with ID: ${micId}`
                  : console.error("unreachable")}
          </Alert>
        )}
        {(!nodes || !nodes.length) && (
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
              No nodes available for sharing
            </Typography>
          </Paper>
        )}
        {/* Content */}
        {nodes && nodes.length > 0 && (
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
        )}
        <Paper sx={{ maxWidth: 500, borderRadius: "0", padding: 1 }}>
          <Grid container justify="space-between">
            <FormControlLabel
              control={
                <Switch
                  onChange={() => {
                    const currentAllDesktopAudio = !allDesktopAudio;
                    setAllDesktopAudio(currentAllDesktopAudio);
                    updateLocalStorage(ALL_DESKTOP, currentAllDesktopAudio);

                    if (currentAllDesktopAudio) {
                      debouncedShareAllDesktopAudio();
                    } else {
                      shareNodes(currentAllDesktopAudio);
                    }
                  }}
                />
              }
              sx={{ marginLeft: 1, marginTop: -1 }}
              label="All Desktop Audio"
              checked={allDesktopAudio}
              disabled={!isHealthy}
            />
            <Button
              sx={{
                minWidth: 75,
                marginLeft: "auto",
              }}
              variant="contained"
              color={isRunning ? "error" : "success"}
              onClick={handleStartStop}
              disabled={!isHealthy}
            >
              {isRunning ? "Stop" : "Start"}
            </Button>
          </Grid>
        </Paper>
      </>
    )
  );
}
