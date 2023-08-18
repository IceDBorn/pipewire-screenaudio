import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";

import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { Button, Checkbox } from "@mui/material";
import Alert from "@mui/material/Alert";

import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";

import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Paper from "@mui/material/Paper";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";

const darkTheme = createTheme({
  palette: {
    mode: "dark",
  },
});

const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";
const EXT_VERSION = browser.runtime.getManifest().version;

function createRows(mediaName, applicationName, serial, checked) {
  return { mediaName, applicationName, serial, checked };
}

function App() {
  const [rows, setRows] = useState([]);
  const [allDesktopAudio, setAllDesktopAudio] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [connectorMissing, setConnectorMissing] = useState(false);
  const [versionMatch, setVersionMatch] = useState(false);
  const [NATIVE_VERSION, setNativeVersion] = useState("");

  useEffect(() => {
    sendMessages("GetNodes", [], onNodesResponse);
    setInterval(() => {
      sendMessages("GetNodes", [], onNodesResponse);
    }, 1000);
  }, []);

  function sendMessages(command, args, evaluationFunction) {
    chrome.runtime
      .sendNativeMessage(MESSAGE_NAME, { cmd: command, args: args })
      .then(evaluationFunction, onError);
  }

  function onVersionResponse(response) {
    const nativeVersion = response.version;
    const extVersionSplit = EXT_VERSION.split(".");
    const nativeVersionSplit = nativeVersion.split(".");
    setVersionMatch(
      extVersionSplit[0] === nativeVersionSplit[0] &&
        extVersionSplit[1] === nativeVersionSplit[1],
    );
    setNativeVersion(nativeVersion);
  }

  function onNodesResponse(response) {
    setRows(
      response.map((element) =>
        createRows(
          element.properties["media.name"],
          element.properties["application.name"],
          element.properties["object.serial"],
          false,
        ),
      ),
    );
  }

  function onError(error) {
    console.error(error);
    setConnectorMissing(true);
  }

  chrome.runtime
    .sendNativeMessage(MESSAGE_NAME, { cmd: "GetVersion", args: [] })
    .then(onVersionResponse, onError);

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      {/* Navbar */}
      <AppBar position="static" sx={{ maxWidth: 500 }}>
        <Toolbar>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Pipewire Screenaudio
          </Typography>
          <Button color="inherit">Settings</Button>
        </Toolbar>
      </AppBar>
      {(isRunning || connectorMissing || !versionMatch) && (
        <Alert
          severity={isRunning ? "info" : "error"}
          color={isRunning ? "info" : "error"}
          sx={{ maxWidth: 500 }}
        >
          {!versionMatch
            ? `Version mismatch! Extension: ${EXT_VERSION}, Native: ${NATIVE_VERSION}`
            : isRunning
            ? "Running with ID: 50"
            : "The native connector is missing or misconfigured"}
        </Alert>
      )}
      <Paper sx={{ maxWidth: 500, borderRadius: 0 }}>
        <FormControlLabel
          control={
            <Switch
              onChange={() => {
                setAllDesktopAudio(!allDesktopAudio);
              }}
            />
          }
          sx={{ marginLeft: 1, marginTop: 1 }}
          label="All Desktop Audio"
          disabled={connectorMissing || !versionMatch}
        />
      </Paper>
      {/* Content */}
      <TableContainer
        component={Paper}
        sx={{
          maxWidth: 500,
          overflow: "scroll",
          minHeight: 100,
          maxHeight: 275,
          borderRadius: 0,
          marginTop: -1,
        }}
      >
        <Table
          sx={{ minWidth: 500, maxWidth: 500 }}
          size="small"
          disabled={connectorMissing || !versionMatch}
        >
          <TableHead
            sx={{
              position: "sticky",
              top: 0,
              zIndex: 10,
              background: "#1e1e1e",
            }}
          >
            <TableRow>
              <TableCell>
                <Checkbox
                  disabled={
                    allDesktopAudio || connectorMissing || !versionMatch
                  }
                  onChange={(event) => {
                    setRows(
                      rows.map((row) => {
                        return { ...row, checked: event.target.checked };
                      }),
                    );
                  }}
                />
              </TableCell>
              <TableCell>Media Name</TableCell>
              <TableCell>Application Name</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row, id) => (
              <TableRow
                key={row.mediaName}
                sx={{ "&:last-child td, &:last-child th": { border: 0 } }}
              >
                <TableCell>
                  <Checkbox
                    onChange={(event) => {
                      setRows(
                        rows.map((row, rowId) => {
                          if (rowId !== id) {
                            return row;
                          }
                          return { ...row, checked: event.target.checked };
                        }),
                      );
                    }}
                    disabled={
                      allDesktopAudio || connectorMissing || !versionMatch
                    }
                    checked={row.checked}
                  />
                </TableCell>
                <TableCell component="th" scope="row">
                  <div
                    style={{
                      overflow: "hidden",
                      width: 200,
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {row.mediaName}
                  </div>
                </TableCell>
                <TableCell>
                  <div
                    style={{
                      overflow: "hidden",
                      width: 160,
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {row.applicationName}
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      <Paper sx={{ maxWidth: 500, borderRadius: "0" }}>
        <Button
          sx={{
            marginLeft: "10rem",
            marginBottom: 2,
            minWidth: 75,
          }}
          variant="contained"
          color={isRunning ? "error" : "success"}
          onClick={() => setIsRunning(!isRunning)}
          disabled={connectorMissing || !versionMatch}
        >
          {isRunning ? "Stop" : "Start"}
        </Button>
        <Button
          sx={{
            marginLeft: "1rem",
            marginBottom: 2,
            minWidth: 75,
          }}
          variant="contained"
          color="error"
          disabled={
            !rows.some((row) => row.checked) ||
            isRunning ||
            connectorMissing ||
            !versionMatch ||
            allDesktopAudio
          }
        >
          Hide
        </Button>
      </Paper>
    </ThemeProvider>
  );
}

const rootEl = document.getElementById("root");
const root = createRoot(rootEl);
root.render(<App />);
