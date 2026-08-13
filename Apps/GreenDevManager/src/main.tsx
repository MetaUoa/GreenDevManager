import React, { Component, type ErrorInfo, type ReactNode } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

class StartupBoundary extends Component<{ children: ReactNode }, { error: string }> {
  state = { error: "" };
  static getDerivedStateFromError(error: unknown) { return { error: String(error) }; }
  componentDidCatch(error: unknown, info: ErrorInfo) { console.error("GreenDev startup error", error, info.componentStack); }
  render() {
    if (!this.state.error) return this.props.children;
    const diagnostics = `GreenDev Manager startup error\n${this.state.error}\nCrash log: %FRAMEWORKS_HOME%\\Logs\\GreenDev\\crash.log`;
    return <main className="startup-failure"><div><span>STARTUP RECOVERY</span><h1>GreenDev Manager 启动异常</h1><p>{this.state.error}</p><code>%FRAMEWORKS_HOME%\Logs\GreenDev\crash.log</code><div><button onClick={() => void navigator.clipboard.writeText(diagnostics)}>复制诊断信息</button><button onClick={() => location.reload()}>重新加载</button></div></div></main>;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <StartupBoundary><App /></StartupBoundary>
  </React.StrictMode>
);
