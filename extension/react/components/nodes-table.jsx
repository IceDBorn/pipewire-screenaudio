import React from "react";

import Table from "@mui/material/Table";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableBody from "@mui/material/TableBody";
import TableRow from "@mui/material/TableRow";
import TableCell from "@mui/material/TableCell";

import Paper from "@mui/material/Paper";
import Checkbox from "@mui/material/Checkbox";

export default function NodesTable({
	hasError,
	allDesktopAudio,
	nodes,
	nodeSelection,
	toggleNodes,
}) {
	const allChecked = nodes.every((node) => nodeSelection.has(node.serial));

	return (
		<TableContainer
			component={Paper}
			sx={{
				minHeight: 100,
				maxHeight: 275,
				borderRadius: 0,
			}}
		>
			<Table
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
								onChange={(event) => toggleNodes(null)}
								checked={allChecked}
							/>
						</TableCell>
						<TableCell>Media</TableCell>
						<TableCell>Application</TableCell>
					</TableRow>
				</TableHead>
				<TableBody>
					{nodes.map((node) => (
						<TableRow
							key={node.mediaName}
							sx={{ "&:last-child td, &:last-child th": { border: 0 } }}
						>
							<TableCell>
								<Checkbox
									onChange={(event) => toggleNodes([node.serial])}
									disabled={allDesktopAudio || hasError}
									checked={nodeSelection.has(node.serial)}
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
									{node.mediaName}
								</div>
							</TableCell>
							<TableCell>
								<div
									style={{
										overflow: "hidden",
										width: 140,
										textOverflow: "ellipsis",
										whiteSpace: "nowrap",
									}}
								>
									{node.applicationName}
								</div>
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
		</TableContainer>
	);
}
