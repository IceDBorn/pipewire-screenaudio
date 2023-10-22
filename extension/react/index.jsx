import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  useSearchParams,
  RouterProvider,
} from "react-router-dom";

import Popup from "./routes/popup";
import Settings from "./routes/settings";

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

  if (search.get("page") === "settings") {
    return <Settings />;
  }
}

function App() {
  return <RouterProvider router={router} basename="/react/dist/index.html" />;
}

const rootEl = document.getElementById("root");
const root = createRoot(rootEl);
root.render(<App />);
