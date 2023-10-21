import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

import Popup from "./routes/popup";

const router = createBrowserRouter([
  {
    path: "/react/dist/index.html",
    element: <Popup />,
  },
]);

function App() {
  return <RouterProvider router={router} basename="/react/dist/index.html" />;
}

const rootEl = document.getElementById("root");
const root = createRoot(rootEl);
root.render(<App />);
