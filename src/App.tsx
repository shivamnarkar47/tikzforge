import { Editor } from "./components/editor";
import { PdfViewer } from "./components/pdf-viewer";
import { useDocumentStore } from "./store/document-store";
import { DEFAULT_TEMPLATE } from "./lib/default-template";
import { formatWindowTitle } from "./lib/window-title";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "./components/ui/resizable";
import { useEffect, useState } from "react";
import { useThemeStore } from "./store/theme-store";
import { useSystemTheme } from "./hooks/use-system-theme";
import { saveState, loadState } from "./lib/persistence";

function App() {
  const { content, setContent, pdfData, filename, setFilename } = useDocumentStore();
  const { theme, setTheme, setSystemTheme, resolvedTheme } = useThemeStore();
  const systemTheme = useSystemTheme();
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

  return (
    <div className="h-screen w-screen flex flex-col">
      <ResizablePanelGroup direction="horizontal" className="flex-1">
        <ResizablePanel defaultSize={50} minSize={20}>
          <Editor value={content} onChange={setContent} />
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={50} minSize={20}>
          <PdfViewer pdfData={pdfData} />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
