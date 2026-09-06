/**
 * Parse a LaTeX compilation log and extract structured errors.
 *
 * LaTeX log format:
 * - Errors start with `!`
 * - Line numbers appear as `l.<number>`
 * - File names appear after `(`
 */
export interface ParsedError {
  file: string;
  line: number;
  message: string;
}

export function parseCompilationLog(log: string): ParsedError[] {
  if (!log.trim()) return [];

  const errors: ParsedError[] = [];
  const lines = log.split("\n");

  let currentError: Partial<ParsedError> | null = null;

  for (const line of lines) {
    // Error start: line begins with !
    if (line.startsWith("!")) {
      if (currentError && currentError.message) {
        errors.push(currentError as ParsedError);
      }
      currentError = {
        file: "",
        line: 0,
        message: line.slice(1).trim(),
      };
    }
    // Line number: l.42
    const lineMatch = line.match(/l\.(\d+)/);
    if (lineMatch) {
      if (!currentError) {
        currentError = { file: "", line: 0, message: "" };
      }
      currentError.line = parseInt(lineMatch[1], 10);
    }
    // File name: (./document.tex
    const fileMatch = line.match(/\(([^)]+\.tex)/);
    if (fileMatch) {
      if (!currentError) {
        currentError = { file: "", line: 0, message: "" };
      }
      currentError.file = fileMatch[1];
    }
  }

  // Push last error if exists (has message, file, or line)
  if (currentError && (currentError.message || currentError.file || currentError.line)) {
    errors.push(currentError as ParsedError);
  }

  return errors;
}
