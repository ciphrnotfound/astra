import * as tf from '@tensorflow/tfjs';

interface CodexModel {
    complete(input: string): Promise<string>;
}

class CodexEngine {
    private root: string;
    private index: any;
    private model: CodexModel | null;
    private memory: any;
    private git: string | null;
    private persona: any;

    constructor() {
        this.root = '.';
        this.index = {};
        this.model = null;
        this.memory = {};
        this.git = null;
        this.persona = {};
    }

    static stripMarkdownFences(text: string): string {
        if (text.startsWith('```')) {
            const pos = text.indexOf('\n');
            if (pos !== -1) {
                text = text.substring(pos + 1);
            }
        }
        if (text.endsWith('```')) {
            const pos = text.lastIndexOf('```');
            if (pos !== -1) {
                text = text.substring(0, pos);
            }
        }
        return text.replace(/\s+/g, ' ');
    }

    new(): CodexEngine {
        const memoryPath = this.root + '/.codex/memory.json';
        const git = tf.GitRepo.discover(this.root).unwrap();
        const persona = tf.Persona.load(this.root);
        this.root = this.root;
        this.index = {};
        this.model = null;
        this.memory = tf.MemoryStore.load(memoryPath);
        this.git = git;
        this.persona = persona;
        return this;
    }

    withRoot(root: string): CodexEngine {
        const memoryPath = root + '/.codex/memory.json';
        const git = tf.GitRepo.discover(root).unwrap();
        const persona = tf.Persona.load(root);
        this.root = root;
        this.index = {};
        this.model = null;
        this.memory = tf.MemoryStore.load(memoryPath);
        this.git = git;
        this.persona = persona;
        return this;
    }

    withModel(root: string, model: CodexModel): CodexEngine {
        const memoryPath = root + '/.codex/memory.json';
        const git = tf.GitRepo.discover(root).unwrap();
        const persona = tf.Persona.load(root);
        this.root = root;
        this.index = {};
        this.model = model;
        this.memory = tf.MemoryStore.load(memoryPath);
        this.git = git;
        this.persona = persona;
        return this;
    }

    setPersona(persona: any): void {
        this.persona = persona;
    }

    setModel(model: CodexModel): void {
        this.model = model;
    }

    handleInput(input: string): Promise<string> {
        const trimmed = input.trim();
        if (trimmed.length === 0) {
            return Promise.resolve('Say something about your codebase to get started.');
        }

        let normalized = trimmed;
        if (normalized.startsWith('›')) {
            normalized = normalized.slice(2).trim().replace(/\s+/g, ' ');
        }

        if (normalized.toLowerCase().startsWith('migrate ')) {
            const tokens = normalized.split(' ');
            if (tokens.length >= 8) {
                for (let i = 0; i < tokens.length; i++) {
                    if (tokens[i] === 'from' && i + 1 < tokens.length) {
                        const fromIndex = i + 1;
                        break;
                    }
                }

                for (let i = fromIndex; i < tokens.length; i++) {
                    if (tokens[i] === 'to' && i + 1 < tokens.length) {
                        const toIndex = i + 1;
                        break;
                    }
                }

                for (let i = toIndex; i < tokens.length; i++) {
                    if (tokens[i] === 'output' && i + 1 < tokens.length) {
                        const outIndex = i + 1;
                        break;
                    }
                }

                if (fromIndex !== undefined && toIndex !== undefined && outIndex !== undefined) {
                    const sourceDir = tokens[1];
                    const fromLang = tokens[fromIndex];
                    const toLang = tokens[toIndex];
                    const outputDir = tokens[outIndex];
                    const useAI = tokens.some((token) => token === '--ai');

                    const config: any = {
                        sourceDir,
                        outputDir,
                        fromLang,
                        toLang,
                        useAI,
                    };

                    if (this.model !== null) {
                        return this.handleInput(this.handleInput(`:migrate ${config.sourceDir} ${config.fromLang} ${config.toLang} ${config.outputDir} ${config.useAI ? '--ai' : ''}`));
                    } else {
                        return Promise.resolve('No language model is configured.');
                    }
                }
            }
        }

        if (this.intentFor(trimmed) === null) {
            if (normalized === ':index') {
                this.buildIndex().then(() => {
                    const stats = this.index.stats();
                    if (stats.fileCount === 0) {
                        this.buildIndex();
                    }
                    const message = `Indexed ${stats.fileCount} files with a total of ${stats.totalLines} lines.`;
                    this.memory.add('index', `${this.root}, ${message}`);
                    return Promise.resolve(message);
                });
            } else if (normalized === ':memory') {
                const recent = this.memory.recent(5);
                if (recent.length === 0) {
                    return Promise.resolve('Memory is empty.');
                }
                let out = '';
                for (const entry of recent) {
                    out += `- [${entry.kind}] ${entry.content}\n`;
                }
                return Promise.resolve(out);
            } else if (normalized === ':files-by-lang') {
                const byLang = this.index.filesByLanguage();
                if (byLang.length === 0) {
                    return Promise.resolve('No files indexed yet. Run :index first.');
                }
                let out = '';
                for (const [lang, count] of byLang) {
                    out += `${lang}: ${count} files\n`;
                }
                return Promise.resolve(out);
            } else if (normalized === ':summary') {
                const stats = this.index.stats();
                const hasGit = this.git !== null;
                const recent = this.memory.recent(5);
                const summary = `Project root: ${this.root}\nIndexed files: ${stats.fileCount}\nTotal lines: ${stats.totalLines}\nGit repository detected: ${hasGit ? 'yes' : 'no'}\nRecent memory:\n${recent.map((entry) => `- [${entry.kind}] ${entry.content}`).join('\n')}`;

                if (this.model !== null) {
                    let prompt = `${summary}\nYou are codex. Summarize this project information for the user:\n`;
                    return this.model.complete(prompt).then((answer) => {
                        this.memory.add('summary', answer);
                        return answer;
                    });
                } else {
                    return Promise.resolve(summary);
                }
            } else if (normalized === ':health') {
                if (this.index.stats().fileCount === 0) {
                    this.buildIndex().then(() => {
                        const teamMgr = new tf.TeamManager(this.root);
                        const report = tf.health.computeHealth(this.root, this.index, this.memory, teamMgr);
                        this.memory.addEvent('health', `Health check: ${report.scores.codeQuality} ${report.scores.testHealth} ${report.scores.crossLangDrift} ${report.scores.securitySurface} ${report.scores.gitHealth} ${report.scores.teamVelocity}`);
                        return report.render();
                    });
                } else {
                    const teamMgr = new tf.TeamManager(this.root);
                    const report = tf.health.computeHealth(this.root, this.index, this.memory, teamMgr);
                    this.memory.addEvent('health', `Health check: ${report.scores.codeQuality} ${report.scores.testHealth} ${report.scores.crossLangDrift} ${report.scores.securitySurface} ${report.scores.gitHealth} ${report.scores.teamVelocity}`);
                    return report.render();
                }
            } else if (normalized.startsWith(':bisect ')) {
                if (this.git !== null) {
                    if (this.model !== null) {
                        return timeTravel.runSemanticBisect(this.git, this.model, normalized.slice(8), 20).then((result) => {
                            const out = `🐛 **Time Travel Debugging Complete** 🐛\nAnalyzed recent ${result.analyzedCount} commits.\n**Suspect Commit Found!**\nCommit: ${result.suspectCommitId} (${result.suspectCommitSummary})\nAuthor: ${result.suspectAuthor}\n**AI Explanation:**\n${result.explanation}\n`;
                            return out;
                        });
                    } else {
                        return Promise.resolve('Semantic bisect requires an LLM to be configured (use Groq).');
                    }
                } else {
                    return Promise.resolve('No git repository detected. Semantic bisect requires git.');
                }
            } else if (normalized.startsWith(':vibe ')) {
                const vibeName = normalized.slice(6).trim();
                this.setPersona(tf.Persona.fromVibe(vibeName));
                return Promise.resolve(`Vibe changed to '${vibeName}'!`);
            } else if (normalized.startsWith(':git-history ')) {
                if (this.git !== null) {
                    const rel = normalized.slice(12);
                    const commits = tf.git.recentCommitsForPath(this.git, rel, 5).then((commits) => {
                        if (commits.length === 0) {
                            return `No recent commits found touching ${rel}`;
                        } else {
                            let out = '';
                            for (const commit of commits) {
                                out +=