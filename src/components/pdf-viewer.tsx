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
      <div className="pdf-viewer flex items-center justify-center h-full text-muted-foreground">
        No PDF loaded
      </div>
    );
  }

  return (
    <div className="pdf-viewer h-full w-full">
      <iframe src={url} className="h-full w-full border-0" title="PDF Viewer" />
    </div>
  );
}
