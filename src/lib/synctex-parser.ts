export interface SynctexRecord {
  page: number;
  x: number;
  y: number;
  line: number;
  column: number;
}

/**
 * Parse SyncTeX text format into records.
 * Format: one record per line as "page x y line column"
 * Comments start with %, other lines are metadata.
 */
export function parseSynctex(content: string): SynctexRecord[] {
  if (!content.trim()) return [];

  const records: SynctexRecord[] = [];
  const lines = content.split("\n");

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("%") || !/^\d/.test(trimmed)) {
      continue;
    }

    const parts = trimmed.split(/\s+/);
    if (parts.length >= 5) {
      records.push({
        page: parseInt(parts[0], 10),
        x: parseInt(parts[1], 10),
        y: parseInt(parts[2], 10),
        line: parseInt(parts[3], 10),
        column: parseInt(parts[4], 10),
      });
    }
  }

  return records;
}

/**
 * Find the source line closest to a given PDF position on a specific page.
 * Uses Manhattan distance to find nearest record.
 */
export function findSourcePosition(
  records: SynctexRecord[],
  page: number,
  x: number,
  y: number
): number | null {
  const pageRecords = records.filter((r) => r.page === page);
  if (pageRecords.length === 0) return null;

  let closest = pageRecords[0];
  let minDist = Math.abs(pageRecords[0].x - x) + Math.abs(pageRecords[0].y - y);

  for (const record of pageRecords.slice(1)) {
    const dist = Math.abs(record.x - x) + Math.abs(record.y - y);
    if (dist < minDist) {
      minDist = dist;
      closest = record;
    }
  }

  return closest.line;
}
