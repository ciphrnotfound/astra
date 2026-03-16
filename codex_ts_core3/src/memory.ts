import * as fs from 'fs';
import * as path from 'path';

export interface MemoryEntry {
    kind: string;
    content: string;
    timestamp: number;
    event?: MemoryEvent;
}

export type MemoryEvent =
    | {
          type: 'IndexSnapshot';
          file_count: number;
          total_lines: number;
          languages: Record<string, number>;
      }
    | {
          type: 'MigrationRun';
          from: string;
          to: string;
          file_count: number;
      }
    | {
          type: 'TeamSession';
          developer: string;
          task_id: string;
          duration_secs: number;
          lines_added: number;
          lines_deleted: number;
      }
    | {
          type: 'WorktreeSnapshot';
          changed_files: number;
          files: string[];
      }
    | {
          type: 'HealthSnapshot';
          scores: HealthScores;
      }
    | {
          type: 'GitCommit';
          id: string;
          summary: string;
          author: string;
          date: string;
      };

export interface HealthScores {
    code_quality: number;
    test_health: number;
    cross_lang_drift: number;
    security_surface: number;
    git_health: number;
    team_velocity: number;
}

const DEFAULT_MAX_ENTRIES = 2000;

export function now_secs(): number {
    return Math.floor(Date.now() / 1000);
}

function normalizeEntry(entry: Partial<MemoryEntry>): MemoryEntry {
    return {
        kind: entry.kind ?? 'unknown',
        content: entry.content ?? '',
        timestamp: entry.timestamp ?? 0,
        event: entry.event,
    };
}

export class MemoryStore {
    entries: MemoryEntry[];
    private storePath?: string;
    private maxEntries: number;

    constructor(entries: MemoryEntry[] = [], storePath?: string, maxEntries = DEFAULT_MAX_ENTRIES) {
        this.entries = entries;
        this.storePath = storePath;
        this.maxEntries = maxEntries;
        this.trimToMax();
    }

    static load(storePath: string): MemoryStore {
        let entries: MemoryEntry[] = [];
        try {
            const raw = fs.readFileSync(storePath, 'utf8');
            const parsed = JSON.parse(raw);
            if (Array.isArray(parsed)) {
                entries = parsed.map((item) => normalizeEntry(item));
            }
        } catch {
            entries = [];
        }
        return new MemoryStore(entries, storePath);
    }

    add(kind: string, content: string): void {
        this.pushEntry({
            kind,
            content,
            timestamp: now_secs(),
        });
    }

    addEvent(kind: string, content: string, event: MemoryEvent): void {
        this.pushEntry({
            kind,
            content,
            timestamp: now_secs(),
            event,
        });
    }

    recent(limit: number): MemoryEntry[] {
        if (limit <= 0) {
            return [];
        }
        return this.entries.slice(Math.max(0, this.entries.length - limit));
    }

    eventsOfKind(kind: string): MemoryEntry[] {
        return this.entries.filter((entry) => entry.kind === kind);
    }

    latestEvent(kind: string): MemoryEntry | undefined {
        for (let i = this.entries.length - 1; i >= 0; i -= 1) {
            const entry = this.entries[i];
            if (entry.kind === kind) {
                return entry;
            }
        }
        return undefined;
    }

    eventsSince(timestamp: number): MemoryEntry[] {
        return this.entries.filter((entry) => entry.timestamp >= timestamp);
    }

    search(query: string, limit: number): MemoryEntry[] {
        const needle = query.trim().toLowerCase();
        if (needle.length === 0 || limit <= 0) {
            return [];
        }
        const matches: MemoryEntry[] = [];
        for (let i = this.entries.length - 1; i >= 0; i -= 1) {
            const entry = this.entries[i];
            if (
                entry.kind.toLowerCase().includes(needle) ||
                entry.content.toLowerCase().includes(needle)
            ) {
                matches.push(entry);
                if (matches.length >= limit) {
                    break;
                }
            }
        }
        return matches;
    }

    private pushEntry(entry: MemoryEntry): void {
        if (this.isDuplicateOfLast(entry)) {
            return;
        }
        this.entries.push(entry);
        this.trimToMax();
        this.save();
    }

    private isDuplicateOfLast(entry: MemoryEntry): boolean {
        const last = this.entries[this.entries.length - 1];
        if (!last) {
            return false;
        }
        return (
            last.kind === entry.kind &&
            last.content === entry.content &&
            JSON.stringify(last.event) === JSON.stringify(entry.event)
        );
    }

    private trimToMax(): void {
        if (this.maxEntries <= 0) {
            return;
        }
        if (this.entries.length > this.maxEntries) {
            this.entries.splice(0, this.entries.length - this.maxEntries);
        }
    }

    private save(): void {
        if (!this.storePath) {
            return;
        }
        fs.mkdirSync(path.dirname(this.storePath), { recursive: true });
        fs.writeFileSync(this.storePath, JSON.stringify(this.entries, null, 2));
    }
}
