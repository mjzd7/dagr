// ⚡ DAGR Interactive Landing Page, Multi-File Codebase Importer & 3D AST Slicer Engine

document.addEventListener('DOMContentLoaded', () => {
    initSimulator();
    initRoiCalculator();
    initClientsGrid();
    initHistoryLedger();
    initGraphVisualizer();
});

// 1. Interactive AST Slicing Simulator & Custom Code Engine
let activeScenario = 'typescript';

function initSimulator() {
    renderSimulatorScenario(activeScenario);
    setupCustomCodeListeners();
}

function selectScenario(scenarioKey) {
    activeScenario = scenarioKey;
    ['typescript', 'python', 'rust', 'custom'].forEach(k => {
        const btn = document.getElementById(`sim-btn-${k}`);
        if (btn) {
            if (k === scenarioKey) {
                btn.className = 'px-4 py-2 rounded-xl bg-emerald-500/20 text-emerald-400 font-semibold text-xs border border-emerald-500/40 shadow-sm transition-all flex items-center space-x-1.5';
            } else {
                btn.className = 'px-4 py-2 rounded-xl text-zinc-400 hover:text-white text-xs border border-transparent transition-all flex items-center space-x-1.5';
            }
        }
    });

    const customPanel = document.getElementById('custom-code-panel');

    if (scenarioKey === 'custom') {
        if (customPanel) customPanel.classList.remove('hidden');
        updateDetectedSymbols();
    } else {
        if (customPanel) customPanel.classList.add('hidden');
        renderSimulatorScenario(scenarioKey);
    }
    
    updateGraphForActiveScenario();
    if (activeGraphMode === '3d') update3DGraphForScenario();
}

function renderSimulatorScenario(key) {
    const data = SIMULATOR_SCENARIOS[key];
    if (!data) return;

    document.getElementById('sim-target-label').innerText = data.target;
    document.getElementById('sim-raw-code').innerText = data.rawFile;
    document.getElementById('sim-sliced-code').innerText = data.slicedFile;

    document.getElementById('sim-raw-tokens').innerText = `${data.rawTokens.toLocaleString()} tokens`;
    document.getElementById('sim-sliced-tokens').innerText = `${data.slicedTokens.toLocaleString()} tokens`;

    const savedTokens = data.rawTokens - data.slicedTokens;
    const pct = ((savedTokens / data.rawTokens) * 100).toFixed(1);
    const usdSaved = ((savedTokens / 1_000_000) * 3.0).toFixed(3);

    document.getElementById('sim-saved-tokens').innerText = `+${savedTokens.toLocaleString()} tokens`;
    document.getElementById('sim-compression-pct').innerText = `-${pct}%`;
    document.getElementById('sim-usd-saved').innerText = `$${usdSaved} / prompt`;
    document.getElementById('sim-latency').innerText = data.latency;
    
    const explanationEl = document.getElementById('sim-explanation-text');
    if (explanationEl) {
        explanationEl.innerText = `Pruned monolithic dependencies & hoisted exact AST type signatures in ${data.latency}. Zero token waste.`;
    }
}

// 2. Custom User Code Slicing Handler
function setupCustomCodeListeners() {
    const codeTextarea = document.getElementById('custom-code-input');
    const langSelect = document.getElementById('custom-lang-select');

    if (codeTextarea) {
        codeTextarea.addEventListener('input', () => {
            updateDetectedSymbols();
        });
    }

    if (langSelect) {
        langSelect.addEventListener('change', () => {
            updateDetectedSymbols();
        });
    }
}

function updateDetectedSymbols() {
    const code = document.getElementById('custom-code-input').value;
    const lang = document.getElementById('custom-lang-select').value;
    const badgeContainer = document.getElementById('detected-symbols-badges');
    
    if (!badgeContainer) return;
    
    const symbols = BrowserAstSlicer.extractSymbols(code, lang);
    if (symbols.length === 0) {
        badgeContainer.innerHTML = `<span class="text-[11px] text-zinc-500 font-mono italic">No functions/classes detected yet. Paste your code below.</span>`;
        return;
    }

    badgeContainer.innerHTML = symbols.slice(0, 6).map(s => `
        <button onclick="setTargetSymbol('${s.name}')" class="px-2 py-0.5 rounded-lg bg-zinc-800 hover:bg-emerald-500/20 text-zinc-300 hover:text-emerald-400 border border-white/10 text-[11px] font-mono transition-colors">
            ⚡ ${s.name}
        </button>
    `).join('');
}

function setTargetSymbol(name) {
    const symbolInput = document.getElementById('custom-symbol-input');
    if (symbolInput) {
        symbolInput.value = name;
        executeCustomSlice();
    }
}

function executeCustomSlice() {
    const rawCode = document.getElementById('custom-code-input').value;
    const lang = document.getElementById('custom-lang-select').value;
    const targetSymbol = document.getElementById('custom-symbol-input').value;

    if (!rawCode.trim()) {
        alert('Please paste some code first!');
        return;
    }

    const result = BrowserAstSlicer.sliceCustomCode(rawCode, targetSymbol, lang);

    // Save into persistent history store
    SlicingHistoryStore.addRecord(result);
    renderHistoryLedger();

    // Render results
    document.getElementById('sim-target-label').innerText = `${result.targetSymbol} (${lang})`;
    document.getElementById('sim-raw-code').innerText = result.rawCode;
    document.getElementById('sim-sliced-code').innerText = result.slicedCode;

    document.getElementById('sim-raw-tokens').innerText = `${result.rawTokens.toLocaleString()} tokens`;
    document.getElementById('sim-sliced-tokens').innerText = `${result.slicedTokens.toLocaleString()} tokens`;

    document.getElementById('sim-saved-tokens').innerText = `+${result.tokensSaved.toLocaleString()} tokens`;
    document.getElementById('sim-compression-pct').innerText = `-${result.compressionPct}%`;
    document.getElementById('sim-usd-saved').innerText = `$${result.usdSaved} / prompt`;
    document.getElementById('sim-latency').innerText = result.latency;

    const explanationEl = document.getElementById('sim-explanation-text');
    if (explanationEl) {
        explanationEl.innerText = result.explanation;
    }

    // Update 2D and 3D Visual Graphs
    if (globalGraphVisualizer) {
        const contracts = result.detectedSymbols
            .filter(s => s.name !== result.targetSymbol)
            .slice(0, 3)
            .map(s => s.name);
        const pruned = ['UnrelatedHelperA', 'UnrelatedHelperB', 'DatabaseClient', 'TaxModule'];
        globalGraphVisualizer.loadScenario(result.targetSymbol, contracts, pruned);
    }
    if (global3DVisualizer && activeGraphMode === '3d') {
        const contracts = result.detectedSymbols
            .filter(s => s.name !== result.targetSymbol)
            .slice(0, 3)
            .map(s => s.name);
        const pruned = ['UnrelatedHelperA', 'UnrelatedHelperB', 'DatabaseClient', 'TaxModule'];
        global3DVisualizer.loadScenario(result.targetSymbol, contracts, pruned);
    }
}

// 3. Multi-File Codebase Ingestion (GitHub, GitLab, ZIP, Folder)
function switchIngestMode(mode) {
    ['github', 'zip', 'folder'].forEach(m => {
        const tab = document.getElementById(`ingest-tab-${m}`);
        const panel = document.getElementById(`ingest-panel-${m}`);
        if (m === mode) {
            if (tab) tab.className = 'px-4 py-2 rounded-xl bg-white/10 text-white font-semibold text-xs border border-white/20 transition-all flex items-center space-x-2';
            if (panel) panel.classList.remove('hidden');
        } else {
            if (tab) tab.className = 'px-4 py-2 rounded-xl text-zinc-400 hover:text-white text-xs border border-transparent transition-all flex items-center space-x-2';
            if (panel) panel.classList.add('hidden');
        }
    });
}

async function importFromGitHubUrl() {
    const urlInput = document.getElementById('github-url-input');
    const statusLabel = document.getElementById('ingest-status-label');
    const importBtn = document.getElementById('github-import-btn');

    const url = urlInput.value.trim();
    if (!url) {
        alert('Please enter a GitHub repository or file URL.');
        return;
    }

    try {
        importBtn.innerText = '⏳ Ingesting...';
        statusLabel.innerText = 'Fetching repository tree & indexing symbols...';
        const result = await globalCodebaseImporter.importFromGitHub(url);

        renderCodebaseResults(result);
        statusLabel.innerText = `✓ Loaded ${result.totalFiles} files (${result.totalSymbols} symbols)`;
        importBtn.innerText = '⚡ Ingest & Index Repo';
    } catch (e) {
        console.error(e);
        statusLabel.innerText = `Error: ${e.message}`;
        alert(e.message);
        importBtn.innerText = '⚡ Ingest & Index Repo';
    }
}

async function handleZipUpload(event) {
    const file = event.target.files[0];
    if (!file) return;

    const statusLabel = document.getElementById('ingest-status-label');
    statusLabel.innerText = `Extracting ${file.name}...`;

    try {
        const result = await globalCodebaseImporter.importFromZip(file);
        renderCodebaseResults(result);
        statusLabel.innerText = `✓ Extracted ${result.totalFiles} files (${result.totalSymbols} symbols) from ZIP`;
    } catch (e) {
        alert(`ZIP Extraction failed: ${e.message}`);
    }
}

async function handleFolderUpload(event) {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    const statusLabel = document.getElementById('ingest-status-label');
    statusLabel.innerText = `Ingesting ${files.length} files...`;

    try {
        const result = await globalCodebaseImporter.importFromFolder(files);
        renderCodebaseResults(result);
        statusLabel.innerText = `✓ Ingested ${result.totalFiles} files (${result.totalSymbols} symbols)`;
    } catch (e) {
        alert(`Folder ingestion failed: ${e.message}`);
    }
}

const SAMPLE_CODEBASES = {
    'expressjs/express': {
        repoName: 'expressjs/express',
        totalFiles: 24,
        files: {
            'lib/router/index.js': `// Express Router Engine
const Route = require('./route');
const Layer = require('./layer');
const methods = require('methods');

function Router(options) {
  const opts = options || {};
  function router(req, res, next) {
    router.handle(req, res, next);
  }
  router.params = {};
  router._params = [];
  router.caseSensitive = opts.caseSensitive;
  router.mergeParams = opts.mergeParams;
  router.strict = opts.strict;
  router.stack = [];
  return router;
}

Router.prototype.route = function route(path) {
  const route = new Route(path);
  const layer = new Layer(path, {}, route.dispatch.bind(route));
  layer.route = route;
  this.stack.push(layer);
  return route;
};

Router.prototype.use = function use(fn) {
  var offset = 0;
  var path = '/';
  if (typeof fn !== 'function') {
    var arg = fn;
    while (Array.isArray(arg) && arg.length !== 0) {
      arg = arg[0];
    }
    if (typeof arg !== 'function') {
      offset = 1;
      path = fn;
    }
  }
  var callbacks = Array.prototype.slice.call(arguments, offset);
  for (var i = 0; i < callbacks.length; i++) {
    var callback = callbacks[i];
    var layer = new Layer(path, {}, callback);
    layer.route = undefined;
    this.stack.push(layer);
  }
  return this;
};

module.exports = Router;`
        }
    },
    'tiangolo/fastapi': {
        repoName: 'tiangolo/fastapi',
        totalFiles: 18,
        files: {
            'fastapi/security/oauth2.py': `# FastAPI Security OAuth2 Module
from typing import Optional, Dict, Any
from pydantic import BaseModel

class OAuth2PasswordRequestForm(BaseModel):
    grant_type: Optional[str] = "password"
    username: str
    password: str
    scope: str = ""
    client_id: Optional[str] = None
    client_secret: Optional[str] = None

class TokenResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
    expires_in: Optional[int] = 3600

def verify_oauth2_credentials(form_data: OAuth2PasswordRequestForm) -> TokenResponse:
    """Verifies OAuth2 password credentials and issues JWT token."""
    if not form_data.username or not form_data.password:
        raise ValueError("Invalid credentials provided")
    return TokenResponse(access_token="eyJhbGciOi...", token_type="bearer")`
        }
    },
    'tokio-rs/tokio': {
        repoName: 'tokio-rs/tokio',
        totalFiles: 32,
        files: {
            'tokio/src/runtime/task/pool.rs': `// Tokio Async Task Worker Pool
pub struct TaskPoolConfig {
    pub max_threads: usize,
    pub queue_depth: usize,
    pub keep_alive_ms: u64,
}

pub struct TaskPoolMetrics {
    pub active_workers: usize,
    pub completed_tasks: u64,
    pub steal_count: u64,
}

pub fn spawn_worker_pool(config: TaskPoolConfig) -> TaskPoolMetrics {
    println!("Initializing Tokio worker pool with {} threads", config.max_threads);
    TaskPoolMetrics {
        active_workers: config.max_threads,
        completed_tasks: 0,
        steal_count: 0,
    }
}`
        }
    }
};

function quickLoadRepo(repoKey) {
    const sample = SAMPLE_CODEBASES[repoKey];
    if (!sample) return;

    globalCodebaseImporter.clear();
    globalCodebaseImporter.activeRepoName = sample.repoName;
    for (const [path, content] of Object.entries(sample.files)) {
        globalCodebaseImporter.addFile(path, content);
    }
    globalCodebaseImporter.indexSymbols();

    const result = {
        totalFiles: sample.totalFiles || Object.keys(sample.files).length,
        totalSymbols: globalCodebaseImporter.symbolIndex.length,
        repoName: sample.repoName,
        filesList: Object.keys(sample.files)
    };

    document.getElementById('github-url-input').value = `https://github.com/${sample.repoName}`;
    document.getElementById('ingest-status-label').innerText = `✓ Loaded ${sample.repoName} (${result.totalSymbols} symbols indexed)`;
    renderCodebaseResults(result);
}

function quickLoadRandomRepo() {
    const keys = Object.keys(SAMPLE_CODEBASES);
    const randomKey = keys[Math.floor(Math.random() * keys.length)];
    quickLoadRepo(randomKey);
}

function renderCodebaseResults(result) {
    const container = document.getElementById('codebase-results-container');
    const titleEl = document.getElementById('ingested-repo-title');
    const countsEl = document.getElementById('ingested-counts-badge');

    if (container) container.classList.remove('hidden');
    if (titleEl) titleEl.innerText = result.repoName;
    if (countsEl) countsEl.innerText = `(${result.totalFiles} files, ${result.totalSymbols} symbols)`;

    // Generate Tailored AI Prompt Suggestions
    generateCodebasePromptSuggestions(result);

    // Render Filterable Symbol Grid
    renderSymbolGrid(globalCodebaseImporter.searchSymbols(''));
}

function generateCodebasePromptSuggestions(result) {
    const container = document.getElementById('chat-suggestions-container');
    if (!container) return;

    const symbols = globalCodebaseImporter.symbolIndex;
    if (symbols.length === 0) {
        container.innerHTML = `<span class="text-xs text-zinc-500 italic">No symbols found to generate suggestions.</span>`;
        return;
    }

    const s1 = symbols[0];
    const s2 = symbols[1] || symbols[0];
    const s3 = symbols[2] || symbols[0];

    const suggestions = [
        {
            icon: '🔍',
            label: `How does ${s1.name} handle inputs & edge cases?`,
            query: `How does ${s1.name} handle inputs and potential error conditions?`,
            symbol: s1
        },
        {
            icon: '⚡',
            label: `Slice ${s2.name} with exact upstream contracts`,
            query: `Extract ${s2.name} with all parameter interfaces hoisted and bloat pruned.`,
            symbol: s2
        },
        {
            icon: '🛡️',
            label: `What dependencies does ${s3.name} rely on?`,
            query: `Analyze all upstream architectural dependencies of ${s3.name}.`,
            symbol: s3
        },
        {
            icon: '📉',
            label: `What monolithic code gets pruned when asking about ${s1.name}?`,
            query: `What monolithic code is pruned when querying ${s1.name} to save tokens?`,
            symbol: s1
        },
        {
            icon: '🧪',
            label: `Generate unit tests for ${s2.name}`,
            query: `Generate isolated unit tests for ${s2.name} with minimal test fixtures.`,
            symbol: s2
        }
    ];

    container.innerHTML = suggestions.map(s => `
        <button onclick="applySuggestion('${s.query.replace(/'/g, "\\'")}', '${s.symbol.file}', '${s.symbol.name}', '${s.symbol.language}')" class="px-3 py-1.5 rounded-xl bg-zinc-900/90 hover:bg-emerald-500/20 text-zinc-300 hover:text-emerald-300 border border-white/10 hover:border-emerald-500/40 text-xs font-mono transition-all text-left flex items-center space-x-1.5 shadow-sm group">
            <span class="group-hover:scale-110 transition-transform">${s.icon}</span>
            <span>${s.label}</span>
        </button>
    `).join('');
}

function applySuggestion(queryText, filePath, symbolName, language) {
    const input = document.getElementById('codebase-chat-input');
    if (input) input.value = queryText;
    sliceCodebaseSymbol(filePath, symbolName, language);
    submitCodebaseChat(queryText, symbolName);
}

function submitCodebaseChat(customQuery = '', targetSymbolName = '') {
    const input = document.getElementById('codebase-chat-input');
    const query = customQuery || (input ? input.value.trim() : '');
    if (!query) {
        alert('Please enter a question or click one of the suggestions above!');
        return;
    }

    const card = document.getElementById('chat-response-card');
    const queryEl = document.getElementById('chat-response-query');
    const textEl = document.getElementById('chat-response-text');
    const statsEl = document.getElementById('chat-response-stats');

    // Find the most relevant symbol for the question
    let sym = null;
    if (targetSymbolName) {
        sym = globalCodebaseImporter.symbolIndex.find(s => s.name === targetSymbolName);
    }
    if (!sym) {
        sym = globalCodebaseImporter.symbolIndex.find(s => query.toLowerCase().includes(s.name.toLowerCase())) || globalCodebaseImporter.symbolIndex[0];
    }

    if (!sym) {
        alert('Please import a codebase first.');
        return;
    }

    const rawContent = globalCodebaseImporter.getFileContent(sym.file);
    const sliceResult = BrowserAstSlicer.sliceCustomCode(rawContent, sym.name, sym.language);

    if (card) card.classList.remove('hidden');
    if (queryEl) queryEl.innerText = `💬 "${query}"`;
    if (statsEl) statsEl.innerText = `🎯 ${sym.name} • ${sliceResult.slicedTokens} tokens (-${sliceResult.compressionPct}% bloat pruned)`;

    if (textEl) {
        textEl.innerHTML = `
<div class="space-y-2">
    <p class="text-zinc-300">
        To answer this query, DAGR analyzed <strong class="text-emerald-400 font-mono">${sym.file}:${sym.line}</strong> and extracted the exact AST slice for <code class="text-cyan-300 bg-zinc-950 px-1.5 py-0.5 rounded">${sym.name}</code>.
    </p>
    <div class="p-3 rounded-lg bg-zinc-950 border border-white/10 font-mono text-[11px] text-emerald-300 overflow-x-auto whitespace-pre">
${sliceResult.slicedCode}
    </div>
    <div class="text-[11px] text-zinc-400 pt-1 flex items-center justify-between border-t border-white/10">
        <span>⚡ <strong>Pruned Bloat:</strong> Unrelated helpers & monolithic SDK bindings omitted.</span>
        <span class="text-cyan-400 font-bold">Latency: ${sliceResult.latency}</span>
    </div>
</div>
        `;
    }

    // Also slice in playground
    sliceCodebaseSymbol(sym.file, sym.name, sym.language);
}

function handleSymbolSearch(event) {
    const query = event.target.value;
    const matches = globalCodebaseImporter.searchSymbols(query);
    renderSymbolGrid(matches);
}

function renderSymbolGrid(symbols) {
    const grid = document.getElementById('codebase-symbols-grid');
    if (!grid) return;

    if (symbols.length === 0) {
        grid.innerHTML = `<div class="col-span-full py-6 text-center text-zinc-500 font-mono text-xs">No matching symbols found. Try searching for "function", "class", or "get".</div>`;
        return;
    }

    grid.innerHTML = symbols.map(s => `
        <div onclick="sliceCodebaseSymbol('${s.file}', '${s.name}', '${s.language}')" class="p-2.5 rounded-xl bg-zinc-900/90 hover:bg-zinc-800 border border-white/10 hover:border-emerald-500/40 cursor-pointer transition-all group flex items-center justify-between">
            <div class="min-w-0">
                <div class="text-xs font-bold text-white group-hover:text-emerald-400 truncate flex items-center space-x-1.5">
                    <span>⚡</span>
                    <span class="truncate">${s.name}</span>
                </div>
                <div class="text-[10px] font-mono text-zinc-500 truncate">${s.file}:${s.line}</div>
            </div>
            <span class="px-2 py-0.5 rounded bg-zinc-800 text-[10px] font-mono text-cyan-400 border border-white/5 uppercase">${s.language}</span>
        </div>
    `).join('');
}

function sliceCodebaseSymbol(filePath, symbolName, language) {
    const rawContent = globalCodebaseImporter.getFileContent(filePath);
    if (!rawContent) return;

    // Populate custom code panel with selected file
    document.getElementById('custom-code-input').value = rawContent;
    document.getElementById('custom-symbol-input').value = symbolName;
    document.getElementById('custom-lang-select').value = language;

    selectScenario('custom');
    executeCustomSlice();
}

// 4. Persistent History & Telemetry Ledger
function initHistoryLedger() {
    renderHistoryLedger();
}

function renderHistoryLedger() {
    const ledgerTable = document.getElementById('history-ledger-body');
    const metrics = SlicingHistoryStore.getMetrics();
    const history = SlicingHistoryStore.getHistory();

    if (document.getElementById('history-total-slices')) {
        document.getElementById('history-total-slices').innerText = metrics.totalSlices;
    }
    if (document.getElementById('history-tokens-saved')) {
        document.getElementById('history-tokens-saved').innerText = `${(metrics.totalTokensSaved / 1000).toFixed(1)}k`;
    }
    if (document.getElementById('history-avg-compression')) {
        document.getElementById('history-avg-compression').innerText = metrics.avgCompression;
    }
    if (document.getElementById('history-usd-saved')) {
        document.getElementById('history-usd-saved').innerText = metrics.totalUsdSaved;
    }

    if (!ledgerTable) return;

    if (history.length === 0) {
        ledgerTable.innerHTML = `<tr><td colspan="6" class="py-8 text-center text-zinc-500 font-mono text-xs">No slicing iterations recorded yet. Paste your code above and click "Slice with DAGR" to track live token cuts!</td></tr>`;
        return;
    }

    ledgerTable.innerHTML = history.map((item, idx) => `
        <tr class="border-b border-white/5 hover:bg-white/[0.02] transition-colors">
            <td class="py-3 px-4 text-zinc-400 font-mono text-xs">#${history.length - idx}</td>
            <td class="py-3 px-4 text-zinc-500 font-mono text-xs">${item.dateFormatted}</td>
            <td class="py-3 px-4 font-mono text-xs">
                <span class="px-2 py-0.5 rounded bg-zinc-800 text-cyan-400 border border-white/10">${item.language}</span>
                <span class="text-white font-semibold ml-1.5">${item.targetSymbol}</span>
            </td>
            <td class="py-3 px-4 font-mono text-xs text-zinc-400">${item.rawTokens.toLocaleString()} → <strong class="text-emerald-400">${item.slicedTokens.toLocaleString()}</strong></td>
            <td class="py-3 px-4 font-mono text-xs text-emerald-400 font-bold">-${item.compressionPct}%</td>
            <td class="py-3 px-4 font-mono text-xs text-indigo-400 font-semibold">$${item.usdSaved}</td>
        </tr>
    `).join('');
}

function clearSlicingHistory() {
    if (confirm('Clear all local slicing iteration records?')) {
        SlicingHistoryStore.clear();
        renderHistoryLedger();
    }
}

// 5. Dynamic Token ROI Financial Calculator
function initRoiCalculator() {
    const teamSlider = document.getElementById('calc-team-size');
    const promptsSlider = document.getElementById('calc-prompts-day');
    const modelSelect = document.getElementById('calc-model-pricing');

    if (!teamSlider || !promptsSlider || !modelSelect) return;

    function recalculate() {
        const teamSize = parseInt(teamSlider.value, 10);
        const promptsPerDay = parseInt(promptsSlider.value, 10);
        const pricePerM = parseFloat(modelSelect.value);

        document.getElementById('calc-team-val').innerText = `${teamSize} engineers`;
        document.getElementById('calc-prompts-val').innerText = `${promptsPerDay} prompts / dev / day`;

        const tokensSavedPerMonth = teamSize * promptsPerDay * 9500 * 21;
        const usdSavedPerMonth = (tokensSavedPerMonth / 1_000_000) * pricePerM;
        const hoursSavedPerYear = Math.round(teamSize * 2.5 * 12);

        document.getElementById('calc-tokens-monthly').innerText = `${(tokensSavedPerMonth / 1_000_000).toFixed(1)}M`;
        document.getElementById('calc-usd-monthly').innerText = `$${Math.round(usdSavedPerMonth).toLocaleString()}`;
        document.getElementById('calc-hours-annual').innerText = `${hoursSavedPerYear.toLocaleString()} hrs`;
    }

    teamSlider.addEventListener('input', recalculate);
    promptsSlider.addEventListener('input', recalculate);
    modelSelect.addEventListener('change', recalculate);
    recalculate();
}

// 6. 31 Supported AI Coding Clients Grid
function initClientsGrid() {
    const grid = document.getElementById('clients-grid-container');
    const searchInput = document.getElementById('clients-search-input');
    if (!grid) return;

    function renderClients(query = '') {
        const q = query.toLowerCase().trim();
        const filtered = CLIENTS_DATA.filter(c => 
            c.name.toLowerCase().includes(q) || 
            c.id.toLowerCase().includes(q) || 
            c.category.toLowerCase().includes(q)
        );

        if (filtered.length === 0) {
            grid.innerHTML = `<div class="col-span-full py-12 text-center text-zinc-500 font-mono text-xs">No matching AI client found for "${query}". Try "cursor", "claude", or "neovim".</div>`;
            return;
        }

        grid.innerHTML = filtered.map(c => `
            <div class="glass-card rounded-2xl p-4 flex flex-col justify-between space-y-3 group hover:border-emerald-500/30 transition-all">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-xl bg-zinc-900/90 border border-white/10 p-2 flex items-center justify-center shrink-0 group-hover:scale-105 transition-transform">
                        <img src="${c.icon}" alt="${c.name}" class="w-6 h-6 object-contain" />
                    </div>
                    <div class="min-w-0">
                        <div class="text-sm font-bold text-white truncate">${c.name}</div>
                        <div class="text-[11px] font-mono text-zinc-400 truncate">${c.category}</div>
                    </div>
                </div>

                <div class="pt-2 border-t border-white/5 space-y-2">
                    <div class="flex items-center justify-between text-[11px] font-mono text-zinc-500 truncate">
                        <span class="truncate">${c.config}</span>
                    </div>
                    <button onclick="copyToClipboard('${c.cmd}', this)" class="w-full py-1.5 px-3 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-white/10 text-xs font-mono text-emerald-400 hover:text-emerald-300 flex items-center justify-between transition-colors">
                        <span class="truncate">${c.cmd}</span>
                        <span class="ml-2 text-zinc-500 shrink-0 copy-icon">📋</span>
                    </button>
                </div>
            </div>
        `).join('');
    }

    if (searchInput) {
        searchInput.addEventListener('input', (e) => renderClients(e.target.value));
    }
    renderClients();
}

// 7. Clipboard Utility
function copyToClipboard(text, triggerBtn) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => showCopyFeedback(triggerBtn));
    } else {
        const textarea = document.createElement('textarea');
        textarea.value = text;
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
        showCopyFeedback(triggerBtn);
    }
}

function showCopyFeedback(btn) {
    if (!btn) return;
    const origHtml = btn.innerHTML;
    btn.innerHTML = `<span class="text-emerald-400 font-bold">✓ Copied to clipboard!</span>`;
    setTimeout(() => {
        btn.innerHTML = origHtml;
    }, 1800);
}
