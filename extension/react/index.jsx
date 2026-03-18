import React from "react";
import { createRoot } from "react-dom/client";
import {
	createBrowserRouter,
	useSearchParams,
	RouterProvider,
} from "react-router-dom";

import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";

import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import Popup from "./routes/popup";

const darkTheme = createTheme({
	palette: {
		mode: "dark",
	},
});

const router = createBrowserRouter([
	{
		path: "/react/dist/index.html",
		element: <PageWrapper />,
	},
]);

function PageWrapper() {
	const [search] = useSearchParams();
	console.log(search);

	if (!search.has("page")) {
		return <Popup />;
	}
}

function App() {
	return (
		<ThemeProvider theme={darkTheme}>
			<CssBaseline />
			<RouterProvider router={router} />
		</ThemeProvider>
	);
}

const rootEl = document.getElementById("root");
const root = createRoot(rootEl);
root.render(<App />);
