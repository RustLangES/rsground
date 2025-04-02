import "./App.sass";
import "../public/fonts/inter.css"

import { Component } from "solid-js";
import { Sidebar } from "./core/sidebar";
import { Panels } from "./core/panels";
import { interceptAuthCallback } from "./core/auth";

const App: Component = () => {
  interceptAuthCallback();

  return (
    <>
      <Sidebar />
      <Panels />
    </>
  );
};

export default App;
