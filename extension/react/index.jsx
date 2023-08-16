import { createRoot } from "react-dom/client";
import { useState } from "react";

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

function createData(name, binary, checked) {
  return { name, binary, checked };
}

function App() {
  const [rows, setRows] = useState([
    createData("AudioCallbackDriver", "Firefox", false),
    createData("Dark Souls III", "DarkSoulsIII.exe", false),
  ]);
  const [allDesktopAudio, setAllDesktopAudio] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [connectorMissing, setConnectorMissing] = useState(false);

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
      {(isRunning || connectorMissing) && (
        <Alert
          severity={isRunning ? "info" : "error"}
          color={isRunning ? "info" : "error"}
          sx={{ maxWidth: 500 }}
        >
          {isRunning
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
          sx={{ marginLeft: "0.5rem", marginTop: 1 }}
          label="All Desktop Audio"
          disabled={connectorMissing}
        />
      </Paper>
      {/* Content */}
      <TableContainer
        component={Paper}
        sx={{
          maxWidth: 500,
          overflow: "scroll",
          maxHeight: 335,
          borderRadius: 0,
        }}
      >
        <Table
          sx={{ minWidth: 500, maxWidth: 500 }}
          size="small"
          disabled={connectorMissing}
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
                <Checkbox disabled={allDesktopAudio || connectorMissing} />
              </TableCell>
              <TableCell>Name</TableCell>
              <TableCell>Binary</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row, id) => (
              <TableRow
                key={row.name}
                sx={{ "&:last-child td, &:last-child th": { border: 0 } }}
              >
                <TableCell>
                  <Checkbox
                    onChange={(event) => {
                      setRows((rows) =>
                        rows.map((row, rowId) => {
                          if (rowId !== id) {
                            return row;
                          }
                          return { ...row, checked: event.target.checked };
                        }),
                      );
                    }}
                    disabled={allDesktopAudio || connectorMissing}
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
                    {row.name}
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
                    {row.binary}
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
            marginTop: 2,
            marginBottom: 2,
            minWidth: 75,
          }}
          variant="contained"
          color={isRunning ? "error" : "success"}
          onClick={() => setIsRunning(!isRunning)}
          disabled={connectorMissing}
        >
          {isRunning ? "Stop" : "Start"}
        </Button>
        <Button
          sx={{
            marginLeft: "1rem",
            marginTop: 2,
            marginBottom: 2,
            minWidth: 75,
          }}
          variant="contained"
          color="error"
          disabled={
            !rows.some((row) => row.checked) ||
            isRunning ||
            connectorMissing ||
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
