import { Editor } from "./components/editor";
import { PdfViewer } from "./components/pdf-viewer";
import { CompileButton } from "./components/compile-button";
import { ErrorList } from "./components/error-list";
import { LogPanel } from "./components/log-panel";
import { FirstLaunchGate } from "./components/first-launch-gate";
import { useDocumentStore } from "./store/document-store";
import { DEFAULT_TEMPLATE } from "./lib/default-template";
import { formatWindowTitle } from "./lib/window-title";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "./components/ui/resizable";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { Toggle } from "./components/ui/toggle";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./components/ui/tooltip";
import { Moon, Sun, Save, ScrollText, FolderOpen } from "lucide-react";
import { useEffect, useState, useCallback } from "react";
import { useThemeStore } from "./store/theme-store";
import { useSystemTheme } from "./hooks/use-system-theme";
import { useCompile } from "./hooks/use-compile";
import { useFirstLaunch } from "./hooks/use-first-launch";
import { saveState, loadState } from "./lib/persistence";
import { useAutoCompile } from "./hooks/use-auto-compile";
import { useKeyboardShortcuts } from "./hooks/use-keyboard-shortcuts";
import { useAutoSave } from "./hooks/use-auto-save";
import { useTabCloseGuard } from "./hooks/use-tab-close-guard";
import { useUpdater } from "./hooks/use-updater";
import { useTabStore } from "./store/tab-store";
import { isTauri } from "./lib/tauri";
import { TEMPLATES } from "./lib/templates";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

function App() {
  const {
    content,
    setContent,
    pdfData,
    filename,
    setFilename,
    isDirty,
    markSaved,
    recentFiles,
    addRecentFile,
    openFile,
  } = useDocumentStore();
  const { theme, setTheme, setSystemTheme, resolvedTheme } = useThemeStore();
  const systemTheme = useSystemTheme();
  const { compile, cancel, phase } = useCompile();
  useFirstLaunch();
  const [ready, setReady] = useState(false);
  const [showLog, setShowLog] = useState(false);
  const [autoCompileEnabled, setAutoCompileEnabled] = useState(true);

  const onCompileCallback = useCallback(() => {
    void compile(filename, content);
  }, [filename, content]);

  const { trigger: triggerAutoCompile } = useAutoCompile({
    onCompile: onCompileCallback,
    enabled: autoCompileEnabled,
    debounceMs: 2000,
  });

  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const addTab = useTabStore((s) => s.addTab);
  const setActiveTab = useTabStore((s) => s.setActiveTab);
  const pendingCloseTabId = useTabStore((s) => s.pendingCloseTabId);
  const { requestClose, forceClose, cancelClose } = useTabCloseGuard();

  const { checking: updateChecking, updateInfo } = useUpdater();

  const handleOpen = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const selected = await openDialog();
      if (typeof selected === "string") {
        const fileContent = await readTextFile(selected);
        openFile(selected, fileContent);
        addRecentFile(selected);
        const newId = addTab(selected, fileContent);
        setActiveTab(newId);
      }
    } catch {
      // best effort
    }
  }, [openFile, addRecentFile, addTab, setActiveTab]);

  const handleOpenRecent = useCallback(
    async (path: string) => {
      if (!isTauri() || !path) return;
      try {
        const fileContent = await readTextFile(path);
        openFile(path, fileContent);
        addRecentFile(path);
      } catch {
        // best effort
      }
    },
    [openFile, addRecentFile]
  );

  const handleSave = useCallback(() => {
    markSaved();
    if (isTauri() && filename) {
      writeTextFile(filename, content).catch(() => {});
    }
  }, [markSaved, filename, content]);

  const handleAutoSave = useCallback(() => {
    if (isDirty) markSaved();
  }, [isDirty, markSaved]);

  useKeyboardShortcuts({ onOpen: handleOpen, onSave: handleSave });

  useAutoSave({ onSave: handleAutoSave, intervalMs: 30000 });

  // Restore persisted state on mount
  useEffect(() => {
    const persisted = loadState<{
      theme: "light" | "dark" | "system";
      autoCompileEnabled?: boolean;
      windowX?: number;
      windowY?: number;
      windowWidth?: number;
      windowHeight?: number;
      dividerPosition?: number;
    }>("tikzforge_state");
    if (persisted?.theme) {
      setTheme(persisted.theme);
    }
    if (typeof persisted?.autoCompileEnabled === "boolean") {
      setAutoCompileEnabled(persisted.autoCompileEnabled);
    }
    setReady(true);
  }, []);

  // Sync system theme
  useEffect(() => {
    setSystemTheme(systemTheme);
  }, [systemTheme]);

  // Apply theme class to document
  const resolved = resolvedTheme();
  useEffect(() => {
    if (!ready) return;
    document.documentElement.classList.toggle("dark", resolved === "dark");
  }, [resolved, ready]);

  // Persist theme + auto-compile changes
  useEffect(() => {
    if (!ready) return;
    saveState("tikzforge_state", { theme, autoCompileEnabled });
  }, [theme, autoCompileEnabled, ready]);

  useEffect(() => {
    if (!content && ready) {
      setContent(DEFAULT_TEMPLATE);
      setFilename("untitled.tex");
    }
  }, [content, setContent, setFilename, ready]);

  useEffect(() => {
    document.title = formatWindowTitle(filename);
  }, [filename]);

  // Initialize a tab on first content
  useEffect(() => {
    if (ready && tabs.length === 0 && content) {
      addTab(filename || "untitled.tex", content);
    }
  }, [ready, tabs.length, content, filename, addTab]);

  if (!ready) return null;

  const isDark = resolved === "dark";

  const handleEditorChange = (value: string) => {
    setContent(value);
    triggerAutoCompile();
  };

  return (
    <TooltipProvider>
      <FirstLaunchGate>
        <div className="h-screen w-screen flex flex-col bg-background text-foreground">
          <header className="flex flex-wrap items-center gap-2 border-b bg-background/80 px-3 py-2 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <span className="text-sm font-semibold shrink-0 tracking-tight">TikzForge</span>
            <Input
              aria-label="Filename"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              className="h-8 max-w-64"
              placeholder="untitled.tex"
            />
            {isDirty && (
              <span className="text-xs text-muted-foreground shrink-0" aria-live="polite">
                ● unsaved
              </span>
            )}
            <select
              aria-label="Templates"
              defaultValue=""
              className="h-8 rounded-md border border-input bg-background px-2 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
              onChange={(e) => {
                const tpl = TEMPLATES.find((t) => t.name === e.target.value);
                if (tpl) setContent(tpl.code);
              }}
            >
              <option value="">Templates</option>
              {TEMPLATES.map((t) => (
                <option key={t.name} value={t.name}>
                  {t.name}
                </option>
              ))}
            </select>
            <select
              aria-label="Recent files"
              defaultValue=""
              className="h-8 rounded-md border border-input bg-background px-2 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
              onChange={(e) => {
                if (e.target.value) void handleOpenRecent(e.target.value);
              }}
            >
              <option value="">Recent files</option>
              {recentFiles.map((f) => (
                <option key={f.path} value={f.path}>
                  {f.path}
                </option>
              ))}
            </select>
            <div className="flex-1" />
            <span aria-label="Update status" className="text-xs text-muted-foreground">
              {updateChecking
                ? "Checking…"
                : updateInfo?.available
                ? "Update available"
                : "Up to date"}
            </span>
            <Toggle
              aria-label="Toggle auto-compile"
              pressed={autoCompileEnabled}
              onPressedChange={setAutoCompileEnabled}
            >
              Auto
            </Toggle>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="outline" size="sm" onClick={() => void handleOpen()}>
                  <FolderOpen />
                  Open
                </Button>
              </TooltipTrigger>
              <TooltipContent>Open a .tex file</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="outline" size="sm" onClick={handleSave}>
                  <Save />
                  Save
                </Button>
              </TooltipTrigger>
              <TooltipContent>Save document to disk</TooltipContent>
            </Tooltip>
            <span aria-label="Compile status" aria-live="polite" className="text-xs text-muted-foreground">
              {phase === "compiling"
                ? "Compiling…"
                : phase === "cancelling"
                ? "Cancelling…"
                : phase === "queued"
                ? "Queued…"
                : "Idle"}
            </span>
            <CompileButton
              onCompile={() => void compile(filename, content)}
              onCancel={() => cancel()}
            />
            <Button
              variant="outline"
              size="sm"
              aria-pressed={showLog}
              onClick={() => setShowLog((v) => !v)}
            >
              <ScrollText />
              Logs
            </Button>
            <Toggle
              aria-label="Toggle theme"
              pressed={isDark}
              onPressedChange={() => setTheme(isDark ? "light" : "dark")}
            >
              {isDark ? <Sun /> : <Moon />}
            </Toggle>
          </header>
          {tabs.length > 0 && (
            <div
              className="flex items-center gap-1 border-b bg-muted/30 px-3 py-1"
              role="tablist"
              aria-label="Open documents"
            >
              {tabs.map((tab) => {
                const active = activeTabId === tab.id;
                return (
                  <div key={tab.id} className="flex items-center gap-0.5">
                    <button
                      type="button"
                      role="tab"
                      aria-selected={active}
                      onClick={() => setActiveTab(tab.id)}
                      className={
                        "rounded-md px-2 py-0.5 text-xs transition-colors " +
                        (active
                          ? "bg-background text-foreground shadow-sm"
                          : "text-muted-foreground hover:bg-background/60 hover:text-foreground")
                      }
                    >
                      {tab.filename}
                    </button>
                    <button
                      type="button"
                      aria-label={`Close ${tab.filename}`}
                      onClick={() => requestClose(tab.id)}
                      className="rounded-md px-1 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
                    >
                      ×
                    </button>
                  </div>
                );
              })}
            </div>
          )}
          {pendingCloseTabId && (
            <div
              className="flex items-center gap-2 border-b bg-destructive/5 px-3 py-1"
              role="alertdialog"
              aria-label="Confirm close tab"
            >
              <span className="text-xs">Close tab with unsaved changes?</span>
              <button
                type="button"
                onClick={() => forceClose(pendingCloseTabId)}
                className="rounded-md bg-destructive px-2 py-0.5 text-xs text-destructive-foreground transition-colors hover:bg-destructive/90"
              >
                Discard
              </button>
              <button
                type="button"
                onClick={cancelClose}
                className="rounded-md border border-input bg-background px-2 py-0.5 text-xs transition-colors hover:bg-accent"
              >
                Cancel
              </button>
            </div>
          )}
          <div className="border-b px-3 py-1">
            <ErrorList />
          </div>
          {showLog && (
            <div className="border-b">
              <LogPanel />
            </div>
          )}
          <Tabs defaultValue="split" className="flex min-h-0 flex-1 flex-col">
            <TabsList className="mx-3 mt-2 w-fit">
              <TabsTrigger value="split">Split</TabsTrigger>
              <TabsTrigger value="editor">Editor</TabsTrigger>
              <TabsTrigger value="preview">Preview</TabsTrigger>
            </TabsList>
            <TabsContent value="split" className="min-h-0 flex-1">
              <ResizablePanelGroup direction="horizontal" className="flex-1">
                <ResizablePanel defaultSize={50} minSize={20}>
                  <div className="h-full min-h-0 overflow-auto">
                    <Editor value={content} onChange={handleEditorChange} />
                  </div>
                </ResizablePanel>
                <ResizableHandle withHandle />
                <ResizablePanel defaultSize={50} minSize={20}>
                  <div className="h-full min-h-0 overflow-auto">
                    <PdfViewer pdfData={pdfData} />
                  </div>
                </ResizablePanel>
              </ResizablePanelGroup>
            </TabsContent>
            <TabsContent value="editor" className="min-h-0 flex-1">
              <div className="h-full min-h-0 overflow-auto px-1">
                <Editor value={content} onChange={handleEditorChange} />
              </div>
            </TabsContent>
            <TabsContent value="preview" className="min-h-0 flex-1">
              <div className="h-full min-h-0 overflow-auto">
                <PdfViewer pdfData={pdfData} />
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </FirstLaunchGate>
    </TooltipProvider>
  );
}

export default App;
