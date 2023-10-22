// TODO: Create settings UI
// TODO: Add nodes sorting
import React, { useEffect, useState } from "react";

import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import Button from "@mui/material/Button";
import Alert from "@mui/material/Alert";

import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";

import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import Paper from "@mui/material/Paper";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";

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
  shareAllDesktopAudio,
} from "../lib/backend";

import {
  MIC_ID,
  ALL_DESKTOP,
  readLocalStorage,
  updateLocalStorage,
} from "../lib/local-storage";
import { useDidUpdateEffect, useLocalStorage } from "../lib/hooks";

import NodesTable from "../components/nodes-table";

const darkTheme = createTheme({
  palette: {
    mode: "dark",
  },
});

function mapNode(node) {
  return {
    mediaName: node.properties["media.name"],
    applicationName: node.properties["application.name"],
    serial: node.properties["object.serial"],
    checked: false,
  };
}

export default function Popup() {
  const [isHealthy, setIsHealthy] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [allDesktopAudio, setAllDesktopAudio] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [nativeVersion, setNativeVersion] = useState("");
  const [extensionVersion, setExtensionVersion] = useState("");
  const [nodes, setNodes] = useState([]);
  const [micId, setMicId] = useLocalStorage(MIC_ID);

  const debouncedSetSharingNodes = useDebouncedCallback(() => {
    setSharingNode(getNodeSerialsToShare());
  }, 1000);

  const debouncedShareAllDesktopAudio = useDebouncedCallback(() => {
    if (allDesktopAudio) {
      shareAllDesktopAudio();
    } else {
      setSharingNode([]);
    }
  }, 1000);

  const getNodeSerialsToShare = () =>
    nodes?.filter((node) => node.checked).map((node) => node.serial);

  useEffect(async () => {
    try {
      const health = await healthCheck();
      setIsHealthy(health);
    } catch (err) {
      if (err.message === ERROR_VERSION_MISMATCH) {
        setNativeVersion(err.cause.nativeVersion);
        setExtensionVersion(err.cause.extensionVersion);
      }

      setIsHealthy(false);
      setIsInitialized(true);
      return;
    }

    let previousNodes = null;
    const nodesReceive = () =>
      getNodes().then((n) => {
        const currentNodesStr = n.toString();
        if (currentNodesStr === previousNodes) return;
        previousNodes = currentNodesStr;
        setNodes(n.map(mapNode));
      });
    nodesReceive();
    const nodesInterval = setInterval(nodesReceive, 2000);

    if (micId) {
      const res = await isPipewireScreenAudioRunning(micId);
      console.log({ res, micId });
      setIsRunning(res);
    }

    setAllDesktopAudio(readLocalStorage(ALL_DESKTOP));

    document.addEventListener(EVENT_MIC_ID_UPDATED, handleMicIdUpdated);
    document.addEventListener(EVENT_MIC_ID_REMOVED, handleMicIdRemoved);

    setIsInitialized(true);

    return () => {
      clearInterval(nodesInterval);
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

  function shareNodes(n, a) {
    setNodes(n);
    if (!isRunning || a) return;
    debouncedSetSharingNodes();
  }

  async function handleStartStop() {
    if (!isRunning) {
      startPipewireScreenAudio();
      if (allDesktopAudio) {
        shareAllDesktopAudio();
      } else {
        setSharingNode(getNodeSerialsToShare());
      }
    } else {
      stopPipewireScreenAudio(micId);
    }
  }

  function openSettingsPage() {
    window.open(window.location.href + "?page=settings");
    window.close();
  }

  return (
    isInitialized && (
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
        <AppBar position="static" sx={{ maxWidth: 500 }}>
          <Toolbar>
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Pipewire Screenaudio
            </Typography>
            <Button color="inherit" onClick={openSettingsPage}>
              Settings
            </Button>
          </Toolbar>
        </AppBar>
        {(isRunning || !isHealthy) && (
          <Alert
            severity={isRunning ? "info" : "error"}
            color={isRunning ? "info" : "error"}
            sx={{ maxWidth: 500 }}
          >
            {!isHealthy
              ? `Version mismatch! Extension: ${extensionVersion}, Native: ${nativeVersion}`
              : isRunning
              ? `Running with ID: ${micId}`
              : "The native connector is missing or misconfigured"}
          </Alert>
        )}

        {!nodes.length && (
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
        {nodes.length > 0 && (
          <NodesTable
            shareNodes={shareNodes}
            nodes={nodes}
            hasError={!isHealthy}
            allDesktopAudio={allDesktopAudio}
          />
        )}
        <Paper sx={{ maxWidth: 500, borderRadius: "0" }}>
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
                    shareNodes(nodes, currentAllDesktopAudio);
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
              marginBottom: 1,
              minWidth: 75,
              marginLeft: 13,
            }}
            variant="contained"
            color={isRunning ? "error" : "success"}
            onClick={handleStartStop}
            disabled={!isHealthy}
          >
            {isRunning ? "Stop" : "Start"}
          </Button>
          <Button
            sx={{
              marginLeft: "1rem",
              marginBottom: 1,
              minWidth: 75,
            }}
            variant="contained"
            color="error"
            disabled={
              isRunning ||
              !nodes.some((node) => node.checked) ||
              !isHealthy ||
              allDesktopAudio
            }
          >
            Hide
          </Button>
        </Paper>
      </ThemeProvider>
    )
  );
}
