// 🌐 DAGR Client-Side Multi-File Codebase Ingestion Engine (GitHub, GitLab, ZIP, Folder)

class CodebaseImporter {
    constructor() {
        this.files = new Map(); // filePath -> fileContent
        this.symbolIndex = [];  // Array<{ name, file, line, signature, language }>
        this.activeRepoName = '';
    }

    clear() {
        this.files.clear();
        this.symbolIndex = [];
        this.activeRepoName = '';
    }

    /**
     * Imports a GitHub repository or specific file link via unauthenticated REST API
     */
    async importFromGitHub(url, token = '') {
        this.clear();
        const parsed = this.parseGitHubUrl(url);
        if (!parsed) {
            throw new Error('Invalid GitHub URL. Use format: https://github.com/owner/repo or file link');
        }

        const headers = {
            'Accept': 'application/vnd.github.v3+json'
        };
        if (token) {
            headers['Authorization'] = `token ${token}`;
        }

        this.activeRepoName = `${parsed.owner}/${parsed.repo}`;

        // If specific file URL provided
        if (parsed.filePath) {
            const rawUrl = `https://raw.githubusercontent.com/${parsed.owner}/${parsed.repo}/${parsed.branch || 'main'}/${parsed.filePath}`;
            const res = await fetch(rawUrl);
            if (!res.ok) throw new Error(`Could not fetch file: HTTP ${res.status}`);
            const content = await res.text();
            this.addFile(parsed.filePath, content);
            this.indexSymbols();
            return {
                totalFiles: 1,
                totalSymbols: this.symbolIndex.length,
                repoName: this.activeRepoName,
                primaryFile: parsed.filePath
            };
        }

        // Fetch repository tree (top-level code files)
        const apiUrl = `https://api.github.com/repos/${parsed.owner}/${parsed.repo}/git/trees/${parsed.branch || 'HEAD'}?recursive=1`;
        const res = await fetch(apiUrl, { headers });
        if (!res.ok) {
            if (res.status === 403) {
                throw new Error('GitHub API rate limit exceeded. Please upload a .ZIP file or provide a GitHub Token.');
            }
            throw new Error(`GitHub API error: HTTP ${res.status}`);
        }

        const data = await res.json();
        if (!data.tree) throw new Error('No files found in GitHub repository tree');

        const supportedExts = ['.ts', '.tsx', '.js', '.jsx', '.py', '.rs', '.go'];
        const codeFiles = data.tree
            .filter(item => item.type === 'blob' && supportedExts.some(ext => item.path.endsWith(ext)))
            .slice(0, 40); // Ingest top 40 code files for instant client performance

        if (codeFiles.length === 0) {
            throw new Error('No TypeScript, Python, Rust, or Go source files found in this repository.');
        }

        // Parallel fetch raw files
        const fetchPromises = codeFiles.map(async (file) => {
            try {
                const rawUrl = `https://raw.githubusercontent.com/${parsed.owner}/${parsed.repo}/${parsed.branch || 'HEAD'}/${file.path}`;
                const fileRes = await fetch(rawUrl);
                if (fileRes.ok) {
                    const text = await fileRes.text();
                    this.addFile(file.path, text);
                }
            } catch (e) {
                console.warn(`Could not load ${file.path}:`, e);
            }
        });

        await Promise.all(fetchPromises);
        this.indexSymbols();

        return {
            totalFiles: this.files.size,
            totalSymbols: this.symbolIndex.length,
            repoName: this.activeRepoName,
            filesList: Array.from(this.files.keys())
        };
    }

    /**
     * Imports source files from a ZIP archive in-memory using JSZip
     */
    async importFromZip(fileBlob) {
        this.clear();
        if (typeof JSZip === 'undefined') {
            throw new Error('JSZip library is not loaded');
        }

        const zip = new JSZip();
        const loadedZip = await zip.loadAsync(fileBlob);
        const supportedExts = ['.ts', '.tsx', '.js', '.jsx', '.py', '.rs', '.go'];

        const promises = [];
        loadedZip.forEach((relativePath, zipEntry) => {
            if (!zipEntry.dir && supportedExts.some(ext => relativePath.endsWith(ext))) {
                if (!relativePath.includes('node_modules/') && !relativePath.includes('.git/') && !relativePath.includes('target/')) {
                    promises.push(
                        zipEntry.async('text').then(content => {
                            this.addFile(relativePath, content);
                        })
                    );
                }
            }
        });

        await Promise.all(promises);
        this.activeRepoName = fileBlob.name.replace(/\.zip$/i, '');
        this.indexSymbols();

        return {
            totalFiles: this.files.size,
            totalSymbols: this.symbolIndex.length,
            repoName: this.activeRepoName,
            filesList: Array.from(this.files.keys())
        };
    }

    /**
     * Imports source files from a local directory drop
     */
    async importFromFolder(fileList) {
        this.clear();
        const supportedExts = ['.ts', '.tsx', '.js', '.jsx', '.py', '.rs', '.go'];

        for (const file of fileList) {
            const path = file.webkitRelativePath || file.name;
            if (supportedExts.some(ext => path.endsWith(ext))) {
                if (!path.includes('node_modules/') && !path.includes('.git/') && !path.includes('target/')) {
                    const content = await file.text();
                    this.addFile(path, content);
                }
            }
        }

        this.activeRepoName = 'Local Workspace Folder';
        this.indexSymbols();

        return {
            totalFiles: this.files.size,
            totalSymbols: this.symbolIndex.length,
            repoName: this.activeRepoName,
            filesList: Array.from(this.files.keys())
        };
    }

    addFile(filePath, content) {
        this.files.set(filePath, content);
    }

    indexSymbols() {
        this.symbolIndex = [];
        for (const [filePath, content] of this.files.entries()) {
            const ext = filePath.split('.').pop().toLowerCase();
            let lang = 'typescript';
            if (ext === 'py') lang = 'python';
            else if (ext === 'rs') lang = 'rust';
            else if (ext === 'go') lang = 'go';

            const symbols = BrowserAstSlicer.extractSymbols(content, lang);
            for (const sym of symbols) {
                this.symbolIndex.push({
                    name: sym.name,
                    file: filePath,
                    line: sym.line,
                    signature: sym.signature,
                    language: lang
                });
            }
        }
    }

    searchSymbols(query = '') {
        const q = query.toLowerCase().trim();
        if (!q) return this.symbolIndex.slice(0, 30);

        return this.symbolIndex
            .filter(s => s.name.toLowerCase().includes(q) || s.file.toLowerCase().includes(q))
            .slice(0, 30);
    }

    getFileContent(filePath) {
        return this.files.get(filePath) || '';
    }

    parseGitHubUrl(url) {
        try {
            const cleaned = url.trim().replace(/^https?:\/\//, '').replace(/\/$/, '');
            // Match github.com/owner/repo or github.com/owner/repo/blob/branch/path/to/file
            const match = cleaned.match(/github\.com\/([^\/]+)\/([^\/]+)(?:\/blob\/([^\/]+)\/(.+))?/);
            if (!match) return null;
            return {
                owner: match[1],
                repo: match[2].replace(/\.git$/, ''),
                branch: match[3] || 'main',
                filePath: match[4] || null
            };
        } catch (e) {
            return null;
        }
    }
}

// Global Codebase Importer Singleton
const globalCodebaseImporter = new CodebaseImporter();
