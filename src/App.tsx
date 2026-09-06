import { Editor } from "./components/editor";
import { PdfViewer } from "./components/pdf-viewer";
import { useDocumentStore } from "./store/document-store";
import { DEFAULT_TEMPLATE } from "./lib/default-template";
import { formatWindowTitle } from "./lib/window-title";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "./components/ui/resizable";
import { useEffect } from "react";

function App() {
  const { content, setContent, pdfData, filename, setFilename } = useDocumentStore();

  useEffect(() => {
    if (!content) {
      setContent(DEFAULT_TEMPLATE);
      setFilename("untitled.tex");
    }
  }, []);

  useEffect(() => {
    document.title = formatWindowTitle(filename);
  }, [filename]);

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
