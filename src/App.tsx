import { Editor } from "./components/editor";
import { PdfViewer } from "./components/pdf-viewer";
import { CompileButton } from "./components/compile-button";
import { ErrorList } from "./components/error-list";
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
import { Moon, Sun, Save } from "lucide-react";
import { useEffect, useState } from "react";
import { useThemeStore } from "./store/theme-store";
import { useSystemTheme } from "./hooks/use-system-theme";
import { useCompile } from "./hooks/use-compile";
import { useFirstLaunch } from "./hooks/use-first-launch";
import { saveState, loadState } from "./lib/persistence";

function App() {
  const { content, setContent, pdfData, filename, setFilename, isDirty, markSaved } = useDocumentStore();
  const { theme, setTheme, setSystemTheme, resolvedTheme } = useThemeStore();
  const systemTheme = useSystemTheme();
  const { compile } = useCompile();
  useFirstLaunch();
  const [ready, setReady] = useState(false);

  // Restore persisted state on mount
  useEffect(() => {
    const persisted = loadState<{
      theme: "light" | "dark" | "system";
      windowX?: number;
      windowY?: number;
      windowWidth?: number;
      windowHeight?: number;
      dividerPosition?: number;
    }>("tikzforge_state");
    if (persisted?.theme) {
      setTheme(persisted.theme);
    }
    setReady(true);
  }, []);

  // Sync system theme
  useEffect(() => {
    setSystemTheme(systemTheme);
  }, [systemTheme]);

  // Apply theme class to document
  useEffect(() => {
    if (!ready) return;
    document.documentElement.classList.toggle("dark", resolvedTheme() === "dark");
  }, [resolvedTheme, ready]);

  // Persist theme changes
  useEffect(() => {
    if (!ready) return;
    saveState("tikzforge_state", { theme });
  }, [theme, ready]);

  useEffect(() => {
    if (!content && ready) {
      setContent(DEFAULT_TEMPLATE);
      setFilename("untitled.tex");
    }
  }, [content, setContent, setFilename, ready]);

  useEffect(() => {
    document.title = formatWindowTitle(filename);
  }, [filename]);

  if (!ready) return null;

  const isDark = resolvedTheme() === "dark";

  return (
    <TooltipProvider>
      <FirstLaunchGate>
        <div className="h-screen w-screen flex flex-col bg-background text-foreground">
          <header className="flex items-center gap-2 border-b px-3 py-2">
            <span className="text-sm font-semibold shrink-0">TikzForge</span>
            <Input
              aria-label="Filename"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              className="h-8 max-w-64"
              placeholder="untitled.tex"
            />
            {isDirty && (
              <span className="text-xs text-muted-foreground shrink-0">● unsaved</span>
            )}
            <div className="flex-1" />
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="outline" size="sm" onClick={markSaved}>
                  <Save />
                  Save
                </Button>
              </TooltipTrigger>
              <TooltipContent>Mark document as saved</TooltipContent>
            </Tooltip>
            <CompileButton onCompile={() => void compile(filename, content)} />
            <Toggle
              aria-label="Toggle theme"
              pressed={isDark}
              onPressedChange={() => setTheme(isDark ? "light" : "dark")}
            >
              {isDark ? <Sun /> : <Moon />}
            </Toggle>
          </header>
          <div className="border-b px-3 py-1">
            <ErrorList />
          </div>
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
                    <Editor value={content} onChange={setContent} />
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
                <Editor value={content} onChange={setContent} />
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
