"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Rocket01Icon,
  CheckmarkCircle02Icon,
  Loading03Icon,
  Tick01Icon,
  Cursor01Icon,
  CursorPointer01Icon,
} from "hugeicons-react";
import { XCircle, AlertTriangle, Loader2 } from "lucide-react";

type Phase =
  | "idle"
  | "move-deploy"
  | "click-deploy"
  | "deploying"
  | "deploy-done"
  | "move-preview"
  | "click-preview"
  | "previewing"
  | "resetting";

interface DemoLogEntry {
  output: string;
  level: "normal" | "info" | "warning" | "error" | "success";
  timestamp: string;
  command?: string;
}

const DEPLOY_LOGS: DemoLogEntry[] = [
  { output: "Starting migration analysis", level: "normal", timestamp: "14:32:01.120" },
  { output: "Loading codebase context...", level: "normal", timestamp: "14:32:01.340", command: "astra analyze" },
  { output: "Detected framework: TypeScript + React", level: "normal", timestamp: "14:32:02.810" },
  { output: "Scanning 247 files for patterns...", level: "normal", timestamp: "14:32:03.005" },
  { output: "Building semantic graph...", level: "normal", timestamp: "14:32:05.442" },
  { output: "Identified 15 core modules", level: "normal", timestamp: "14:32:08.500" },
  { output: "Running migration: TypeScript → Rust", level: "normal", timestamp: "14:32:09.100", command: "astra migrate" },
  { output: "Analyzing type dependencies...", level: "normal", timestamp: "14:32:12.330" },
  { output: "Mapping React patterns to Rust equivalents", level: "normal", timestamp: "14:32:18.710" },
  { output: "Generating Rust code (15/15 modules)", level: "normal", timestamp: "14:32:22.200" },
  { output: "Validating generated code...", level: "normal", timestamp: "14:32:25.550" },
  { output: "Running type checker...", level: "normal", timestamp: "14:32:27.880" },
  { output: "All type checks passed", level: "info", timestamp: "14:32:29.200" },
  { output: "Generating migration report...", level: "normal", timestamp: "14:32:30.610" },
  { output: "Migration complete", level: "success", timestamp: "14:32:31.100" },
];

export default function HeroDemo() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [showPreview, setShowPreview] = useState(false);
  const [logs, setLogs] = useState<DemoLogEntry[]>([]);
  const [logIndex, setLogIndex] = useState(0);
  const [cursorClicking, setCursorClicking] = useState(false);
  const [deployStatus, setDeployStatus] = useState<"idle" | "running" | "success">("idle");
  const [cycleKey, setCycleKey] = useState(0);

  const containerRef = useRef<HTMLDivElement>(null);
  const deployBtnRef = useRef<HTMLDivElement>(null);
  const previewBtnRef = useRef<HTMLDivElement>(null);
  const [cursorXY, setCursorXY] = useState({ x: 0, y: 0 });

  const getTargetPos = useCallback(
    (el: HTMLElement | null, offsetX = 0.5, offsetY = 0.5) => {
      if (!el || !containerRef.current) return null;
      const cRect = containerRef.current.getBoundingClientRect();
      const tRect = el.getBoundingClientRect();
      return {
        x: tRect.left - cRect.left + tRect.width * offsetX,
        y: tRect.top - cRect.top + tRect.height * offsetY,
      };
    },
    [],
  );

  const moveCursorTo = useCallback(
    (el: HTMLElement | null, offsetX = 0.5, offsetY = 0.5) => {
      const pos = getTargetPos(el, offsetX, offsetY);
      if (pos) setCursorXY(pos);
    },
    [getTargetPos],
  );

  useEffect(() => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    setCursorXY({ x: rect.width * 0.65, y: rect.height * 0.5 });
  }, [cycleKey]);

  const advancePhase = useCallback((next: Phase, delay: number) => {
    const t = setTimeout(() => setPhase(next), delay);
    return () => clearTimeout(t);
  }, []);

  useEffect(() => {
    let cleanup: (() => void) | undefined;

    switch (phase) {
      case "idle":
        cleanup = advancePhase("move-deploy", 800);
        break;
      case "move-deploy":
        moveCursorTo(deployBtnRef.current);
        cleanup = advancePhase("click-deploy", 800);
        break;
      case "click-deploy":
        setCursorClicking(true);
        setTimeout(() => setCursorClicking(false), 250);
        setDeployStatus("running");
        setLogs([]);
        setLogIndex(0);
        cleanup = advancePhase("deploying", 350);
        break;
      case "deploying":
        break;
      case "deploy-done":
        setDeployStatus("success");
        if (containerRef.current) {
          const rect = containerRef.current.getBoundingClientRect();
          setCursorXY({ x: rect.width * 0.65, y: rect.height * 0.5 });
        }
        cleanup = advancePhase("move-preview", 1200);
        break;
      case "move-preview":
        moveCursorTo(previewBtnRef.current);
        cleanup = advancePhase("click-preview", 800);
        break;
      case "click-preview":
        setCursorClicking(true);
        setTimeout(() => setCursorClicking(false), 250);
        setShowPreview(true);
        cleanup = advancePhase("previewing", 350);
        break;
      case "previewing":
        cleanup = advancePhase("resetting", 3000);
        break;
      case "resetting":
        setShowPreview(false);
        setDeployStatus("idle");
        setLogs([]);
        setLogIndex(0);
        setCycleKey((k) => k + 1);
        cleanup = advancePhase("idle", 600);
        break;
    }
    return cleanup;
  }, [phase, advancePhase, moveCursorTo]);

  useEffect(() => {
    if (phase !== "deploying") return;
    if (logIndex >= DEPLOY_LOGS.length) {
      setPhase("deploy-done");
      return;
    }
    const t = setTimeout(() => {
      setLogs((prev) => [...prev, DEPLOY_LOGS[logIndex]]);
      setLogIndex((i) => i + 1);
    }, 320);
    return () => clearTimeout(t);
  }, [phase, logIndex]);

  const isPointing =
    phase === "move-deploy" ||
    phase === "click-deploy" ||
    phase === "move-preview" ||
    phase === "click-preview";

  return (
    <div key={cycleKey} ref={containerRef} className="relative w-full">
      <div className="rounded-xl bg-white border border-gray-200 shadow-2xl overflow-hidden">
        {/* Title bar */}
        <div className="flex items-center justify-between px-4 py-2.5 bg-gray-50 border-b border-gray-200">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-[#FF5F57]" />
            <span className="w-3 h-3 rounded-full bg-[#FEBC2E]" />
            <span className="w-3 h-3 rounded-full bg-[#28C840]" />
          </div>
          <span className="text-[11px] text-gray-500 font-cabinet">astra.dev</span>
          <div className="w-12" />
        </div>

        {/* Content */}
        <div className="relative h-[22rem] md:h-[26rem] overflow-hidden">
          <AnimatePresence mode="wait">
            {!showPreview ? (
              <DeployPane
                key="deploy"
                logs={logs}
                status={deployStatus}
                isDeploying={phase === "deploying"}
                deployBtnRef={deployBtnRef}
                previewBtnRef={previewBtnRef}
              />
            ) : (
              <PreviewPane key="preview" />
            )}
          </AnimatePresence>
        </div>
      </div>

      {/* Cursor */}
      <motion.div
        className="absolute z-50 pointer-events-none"
        animate={{
          x: cursorXY.x,
          y: cursorXY.y,
          scale: cursorClicking ? 0.8 : 1,
        }}
        transition={{ type: "spring", stiffness: 100, damping: 18, mass: 0.9 }}
        style={{ top: 0, left: 0 }}
      >
        {isPointing ? (
          <CursorPointer01Icon size={28} className="text-white drop-shadow-lg" />
        ) : (
          <Cursor01Icon size={28} className="text-white drop-shadow-lg" />
        )}
        <AnimatePresence>
          {cursorClicking && (
            <motion.div
              initial={{ scale: 0, opacity: 0.6 }}
              animate={{ scale: 2.5, opacity: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.4 }}
              className="absolute top-1 left-1 w-4 h-4 rounded-full bg-[#FF63F9]/40"
            />
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

/* Deploy Pane */
function DeployPane({
  logs,
  status,
  isDeploying,
  deployBtnRef,
  previewBtnRef,
}: {
  logs: DemoLogEntry[];
  status: "idle" | "running" | "success";
  isDeploying: boolean;
  deployBtnRef: React.RefObject<HTMLDivElement | null>;
  previewBtnRef: React.RefObject<HTMLDivElement | null>;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -10 }}
      transition={{ duration: 0.2 }}
      className="h-full flex flex-col"
    >
      <div className="px-5 pt-4 pb-3 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-semibold text-gray-900">typescript-to-rust</h3>
            <p className="text-[11px] text-gray-500 mt-0.5">main branch &middot; TypeScript → Rust</p>
          </div>
          <div className="flex items-center gap-2">
            {status === "running" && (
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-amber-500/10 border border-amber-500/20">
                <Loading03Icon size={12} className="text-amber-400 animate-spin" />
                <span className="text-[10px] font-medium text-amber-400">Migrating</span>
              </div>
            )}
            {status === "success" && (
              <motion.div
                ref={previewBtnRef}
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-[#FF63F9]/20 border border-[#FF63F9]/30 cursor-pointer"
              >
                <CheckmarkCircle02Icon size={12} className="text-[#FF63F9]" />
                <span className="text-[10px] font-semibold text-[#FF63F9]">Preview</span>
              </motion.div>
            )}
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-hidden relative">
        {status === "idle" ? (
          <div className="h-full flex flex-col items-center justify-center gap-3 text-gray-400">
            <Rocket01Icon size={32} />
            <p className="text-xs">Ready to migrate</p>
            <div
              ref={deployBtnRef}
              className="mt-2 px-5 py-2 rounded-full bg-gray-900 text-white text-xs font-semibold"
            >
              Start Migration
            </div>
          </div>
        ) : (
          <div className="font-mono text-xs overflow-y-auto h-full py-2">
            {logs.map((log, i) => {
              const isLast = i === logs.length - 1 && isDeploying;
              const isSuccess = log.level === "success";

              const textClass =
                log.level === "error"
                  ? "text-red-600"
                  : log.level === "warning"
                    ? "text-yellow-600"
                    : log.level === "info"
                      ? "text-blue-600"
                      : isSuccess
                        ? "text-emerald-600"
                        : "text-gray-700";

              const bgClass =
                log.level === "error"
                  ? "bg-red-500/20"
                  : log.level === "warning"
                    ? "bg-yellow-500/20"
                    : "";

              return (
                <motion.div
                  key={i}
                  initial={{ opacity: 0, y: 4 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.15 }}
                  className={`group flex items-start gap-3 px-3 py-1 hover:bg-gray-50 ${bgClass}`}
                >
                  <div className="w-20 shrink-0 flex items-center gap-1 text-[11px] text-gray-500">
                    <span>{log.timestamp}</span>
                    {isLast && !isSuccess && (
                      <Loader2 className="w-3 h-3 text-amber-400 animate-spin" />
                    )}
                    {log.level === "error" && (
                      <XCircle className="w-3 h-3 text-red-400" />
                    )}
                    {log.level === "warning" && (
                      <AlertTriangle className="w-3 h-3 text-yellow-400" />
                    )}
                  </div>
                  <div className={`flex-1 ${textClass}`}>
                    {log.command ? (
                      <>
                        <span className="text-blue-600 font-medium">{log.command}</span>
                        <span className="text-gray-400"> — </span>
                        <span>{log.output}</span>
                      </>
                    ) : (
                      <span>{log.output}</span>
                    )}
                  </div>
                </motion.div>
              );
            })}
          </div>
        )}
      </div>
    </motion.div>
  );
}

/* Preview Pane */
function PreviewPane() {
  return (
    <motion.div
      initial={{ opacity: 0, x: 10 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 10 }}
      transition={{ duration: 0.2 }}
      className="h-full flex flex-col"
    >
      {/* URL bar */}
      <div className="px-4 pt-3 pb-2">
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-50 border border-gray-200">
          <Tick01Icon size={12} className="text-emerald-600" />
          <span className="text-[11px] font-cabinet text-gray-600">astra.dev/migrations/ts-to-rust</span>
        </div>
      </div>

      {/* Simple centered preview */}
      <div className="flex-1 mx-4 mb-4 rounded-lg overflow-hidden border border-gray-200 bg-gray-50 flex items-center justify-center">
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.1 }}
          className="text-center"
        >
          <CheckmarkCircle02Icon size={28} className="text-emerald-600 mx-auto mb-3" />
          <h2 className="text-lg font-bold text-gray-900 mb-1">
            Migration Successful
          </h2>
          <p className="text-xs text-gray-600">
            Your code has been successfully migrated to Rust.
          </p>
        </motion.div>
      </div>
    </motion.div>
  );
}