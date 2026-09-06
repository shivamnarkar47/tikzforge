import { useEffect, useState } from "react";

interface PdfViewerProps {
  pdfData: Uint8Array | null;
}

export function PdfViewer({ pdfData }: PdfViewerProps) {
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    if (!pdfData) {
      setUrl(null);
      return;
    }

    const blob = new Blob([pdfData.buffer as ArrayBuffer], { type: "application/pdf" });
    const objectUrl = URL.createObjectURL(blob);
    setUrl(objectUrl);

    return () => URL.revokeObjectURL(objectUrl);
  }, [pdfData]);

  if (!url) {
    return (
      <div className="pdf-viewer flex h-full flex-col items-center justify-center gap-2 bg-muted/30 p-6 text-center">
        <p className="text-sm font-medium">No PDF loaded</p>
        <p className="text-xs text-muted-foreground">
          Press Compile to build your document and preview it here.
        </p>
      </div>
    );
  }

  return (
    <div className="pdf-viewer h-full w-full">
      <iframe src={url} className="h-full w-full border-0" title="PDF Viewer" />
    </div>
  );
}
