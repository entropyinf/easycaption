import { ReactNode } from "react";
import "./App.css";
import Caption from "./pages/Caption";
import Layout from "./pages/Layout";

const routes: Record<string, ReactNode> = {
  caption: <Caption />,
  setting: <Layout />,
}

function App() {
  const pageId = window.location.hash.substring(1);

  return (
    <>
      {routes[pageId] || <Layout />}
    </>
  );
}

export default App;