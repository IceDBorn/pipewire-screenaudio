import React, { useEffect, useState } from "react";

import Table from "@mui/material/Table";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableBody from "@mui/material/TableBody";
import TableRow from "@mui/material/TableRow";
import TableCell from "@mui/material/TableCell";

import Paper from "@mui/material/Paper";
import Checkbox from "@mui/material/Checkbox";

import { useDebouncedCallback } from "use-debounce";

import {
  SELECTED_ROWS,
  readLocalStorage,
  updateLocalStorage,
} from "../lib/local-storage";

import matchNode from "../lib/match-node";

export default function NodesTable({
  hasError,
  allDesktopAudio,
  nodes,
  shareNodes,
}) {
  const [allChecked, setAllChecked] = useState(false);
  const [rows, setRows] = useState(nodes);

  const debouncedSharedNodes = useDebouncedCallback(shareNodes, 500);

  function onCheckboxChanged(event, id) {
    const isChecked = event.target.checked;
    if (id !== null) {
      setRows(
        rows.map((row, idx) =>
          idx === id ? { ...row, checked: isChecked } : row,
        ),
      );
    } else {
      setRows(rows.map((row) => ({ ...row, checked: isChecked })));
    }
  }

  useEffect(() => {
    const rowsMap = Object.fromEntries(rows.map((r) => [r.serial, r]));

    const saved = readLocalStorage(SELECTED_ROWS);
    if (saved) {
      saved.forEach((s) => {
        const row = rowsMap[s.serial];
        if (row && matchNode(s, row)) {
          row.checked = !!s.checked;
        }
      });
    }

    setRows(Object.values(rowsMap));
  }, []);

  useEffect(() => {
    setAllChecked(rows.map(({ checked }) => checked).every(Boolean));
    updateLocalStorage(SELECTED_ROWS, rows);
    debouncedSharedNodes(rows);
  }, [rows]);

  return (
    <TableContainer
      component={Paper}
      sx={{
        maxWidth: 500,
        overflow: "scroll",
        minHeight: 100,
        maxHeight: 275,
        borderRadius: 0,
      }}
    >
      <Table
        sx={{ minWidth: 500, maxWidth: 500 }}
        size="small"
        disabled={hasError}
      >
        <TableHead
          sx={{
            position: "sticky",
            top: 0,
            zIndex: 10,
            background: "#1e1e1e",
            borderBottom: "solid",
            borderColor: "#515151",
          }}
        >
          <TableRow>
            <TableCell>
              <Checkbox
                disabled={allDesktopAudio || hasError}
                onChange={(event) => onCheckboxChanged(event, null)}
                checked={allChecked}
              />
            </TableCell>
            <TableCell>Media</TableCell>
            <TableCell>Application</TableCell>
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
                  onChange={(event) => onCheckboxChanged(event, id)}
                  disabled={allDesktopAudio || hasError}
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
  );
}
