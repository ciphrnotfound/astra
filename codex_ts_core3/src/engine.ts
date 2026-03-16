interface Result<T> {
    ok: boolean;
    error?: any;
    value?: T;
}

class CodeIndex {
    private files: Map<string, string>;
    private stats: Stats;

    constructor() {
        this.files = new Map();
        this.stats = {
            fileCount: 0,
            totalLines: 0,
        };
    }

    addFile(path: string, contents: string) {
        this.files.set(path, contents);
        this.stats.fileCount++;
        this.stats.totalLines += contents.split("\n").length;
    }

    stats(): Stats {
        return this.stats;
    }
}

interface Stats {
    fileSize: number;
    totalLines: number;
}

interface Persona {
    systemPrompt(): string;
}

class PersonaImpl implements Persona {
    private vibe: string;

    constructor(vibe: string) {
        this.vibe = vibe;
    }

    systemPrompt(): string {
        return this.vibe;
    }
}

class MigrationConfig {
    sourceDir: string;
    output: string;
    fromLang: string;
    toLang: string;
    useAi: boolean;

    constructor(sourceDir: string, output: string, fromLang: string, toLang: string, useAi: boolean) {
        this.sourceDir = sourceDir;
        this.output = output;
        this.fromLang = fromLang;
        this.toLang = toLang;
        this.useAi = useAi;
    }
}

class CodexModel {
    complete(prompt: string): string {
        return prompt;
    }
}

interface MemoryStore {
    load(path: string): MemoryStore;
    recent(limit: number): MemoryEntry[];
    add(key: string, value: string): void;
    addEvent(key: string, value: string, memoryEvent: MemoryEvent): void;
}

class MemoryStoreImpl implements MemoryStore {
    private entries: Map<string, string>;

    constructor() {
        this.entries = new Map();
    }

    load(path: string): MemoryStore {
        // load from file
    }

    recent(limit: number): MemoryEntry[] {
        return Array.from(this.entries).slice(0, limit).map((entry) => ({ kind: entry[0], content: entry[1] }));
    }

    add(key: string, value: string): void {
        this.entries.set(key, value);
    }

    addEvent(key: string, value: string, memoryEvent: MemoryEvent): void {
        this.add(key, value);
    }
}

interface MemoryEvent {
    healthSnapshot: HealthSnapshot;
}

interface HealthSnapshot {
    scores: Scores;
}

interface Scores {
    codeQuality: number;
    testHealth: number;
    crossLangDrift: number;
    securitySurface: number;
    gitHealth: number;
    teamVelocity: number;
}

class MemoryEntry {
    kind: string;
    content: string;
}

class GitRepo {
    private root: string;

    constructor(root: string) {
        this.root = root;
    }

    discover(root: string): string {
        // discover git repo
    }
}

class CodexEngine {
    root: string;
    index: CodeIndex;
    model: CodexModel | null;
    memory: MemoryStore;
    git: string | null;
    persona: Persona;

    constructor() {
        this.root = ".";
        this.index = new CodeIndex();
        this.model = null;
        this.memory = new MemoryStoreImpl();
        this.git = null;
        this.persona = new PersonaImpl("");
    }

    new(): CodexEngine {
        this.loadDefaultSettings();
        return this;
    }

    withRoot(root: string): CodexEngine {
        this.root = root;
        this.loadDefaultSettings();
        return this;
    }

    withModel(model: CodexModel, root: string): CodexEngine {
        this.model = model;
        this.root = root;
        this.loadDefaultSettings();
        return this;
    }

    setPersona(persona: Persona) {
        this.persona = persona;
    }

    setModel(model: CodexModel) {
        this.model = model;
    }

    handleInput(input: string): Result<string> {
        const trimmed = input.trim();
        if (trimmed.length === 0) {
            return {
                ok: true,
                value: "Say something about your codebase to get started.",
            };
        }

        let mut normalized = trimmed;
        if ( normalized.startsWith('›')) {
            normalized = normalized.trimStart().trimStart().slice(1);
        }

        if (normalized.toLowerCase().startsWith("migrate ")) {
            const tokens = normalized.split(" ");
            const args = new Map<string, string>();
            for (const token of tokens) {
                if (token === "from") {
                    args.set("from_LANG", tokens[tokens.indexOf(token) + 1]);
                } else if (token === "to") {
                    args.set("to", tokens[tokens.indexOf(token) + 1]);
                } else if (token === "output") {
                    args.set("output", tokens[tokens.indexOf(token) + 1]);
                } else if (token === "--ai") {
                    args.set("use_ai", "true");
                }
            }

            if (args.size === 4) {
                const from = args.get("from_LANG");
                const to = args.get("to");
                const output = args.get("output");

                return this.handleInput(`:migrate ${args.get("from_LANG")} ${args.get("to")} ${args.get("output")}`);
            }
        }

        if (normalized === ":index") {
            this.buildIndex();
            const stats = this.index.stats();
            const message = `Indexed ${stats.fileCount} files with a total of ${stats.totalLines} lines.`;
            this.memory.add("index", message);
            return {
                ok: true,
                value: message,
            };
        }

        if (normalized === ":memory") {
            const recent = this.memory.recent(5);
            if (recent.length === 0) {
                return {
                    ok: true,
                    value: "Memory is empty.",
                };
            }

            const output = recent.map((entry) => `- [${entry.kind}] ${entry.content}`).join('\n');
            return {
                ok: true,
                value: output,
            };
        }

        if (normalized === ":files-by-lang") {
            const byLang = this.index.files;
            if (byLang.size === 0) {
                return {
                    ok: true,
                    value: "No files indexed yet. Run :index first.",
                };
            }

            const output = Array.from(byLang, ([key, value]) => `${key}: ${value}`).join('\n');
            return {
                ok: true,
                value: output,
            };
        }

        if (normalized === ":summary") {
            const stats = this.index.stats();
            const hasGit = this.git !== null;
            const recent = this.memory.recent(5);

            const output = [
                `Project root: ${this.root}`,
                `Indexed files: ${stats.fileCount}`,
                `Total lines: ${stats.totalLines}`,
                `Git repository detected: ${hasGit}`,
            ].join('\n');

            if (recent.length > 0) {
                output += '\n';
                output += "Recent memory:\n";
                recent.forEach((entry) => {
                    output += `- [${entry.kind}] ${entry.content}\n`;
                });
            }

            if (this.model !== null) {
                const prompt = `You are codex. Summarize this project information for the user:\n${output}`;

                const answer = this.model.complete(prompt);
                this.memory.add("summary", answer);
                return {
                    ok: true,
                    value: answer,
                };
            }

            return {
                ok: true,
                value: output,
            };
        }

        if (normalized === ":health") {
            if (this.index.stats().fileCount === 0) {
                this.buildIndex();
            }

            const teamMgr = new TeamManager(this.root);
            const report = health.computeHealth(this.root, this.index, this.memory, teamMgr);

            this.memory.addEvent("health", report.render(), {
                healthSnapshot: report.scores,
            });
            return {
                ok: true,
                value: report.render(),
            };
        }

        if (normalized.startsWith(":bisect ")) {
            const desc = normalized.slice(9);
            if (this.git !== null) {
                if (this.model !== null) {
                    try {
                        const result = timeTravel.runSemanticBisect(this.git, this.model, desc, 20);
                        const output = `🐛 **Time Travel Debugging Complete** 🐛\n`;
                        output += `Analyzed recent ${result.analyzedCount} commits.\n`;
                        output += `\n**Suspect Commit Found!**\n`;
                        output += `Commit: ${result.suspectCommitID} (${result.suspectCommitSummary})\n`;
                        output += `Author: ${result.suspectAuthor}\n`;
                        output += `\n**AI Explanation:**\n${result.explanation}`;
                        this.memory.add("bisect", output);
                        return {
                            ok: true,
                            value: output,
                        };
                    } catch (e) {
                        return {
                            ok: false,
                            error: e,
                        };
                    }
                } else {
                    return {
                        ok: true,
                        value: "Semantic bisect requires an LLM to be configured (use Groq).",
                    };
                }
            } else {
                return {
                    ok: true,
                    value: "No git repository detected. Semantic bisect requires git.",
                };
            }
        }

        if (normalized.startsWith(":vibe ")) {
            const vibeName = normalized