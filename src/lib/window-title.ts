const APP_NAME = "TikzForge";

export function formatWindowTitle(filename: string): string {
  if (!filename) return APP_NAME;
  return `${filename} — ${APP_NAME}`;
}
