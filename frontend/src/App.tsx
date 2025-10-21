import { ReactNode } from "react";
import "./App.css";
import Settings from "./pages/setting";
import Caption from "./pages/caption";

const routes: Record<string, ReactNode> = {
  caption: <Caption />,
  setting: <Settings />,
}

function App() {
  const pageId = window.location.hash.substring(1);

  return (
    <>
      {routes[pageId] || <Settings />}
    </>
  );
}

export default App;
