// ⚡ DAGR Interactive Landing Page & Custom Code AST Slicer Engine

document.addEventListener('DOMContentLoaded', () => {
    initSimulator();
    initRoiCalculator();
    initClientsGrid();
    initHistoryLedger();
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
    const presetPanel = document.getElementById('preset-playground-panel');

    if (scenarioKey === 'custom') {
        if (customPanel) customPanel.classList.remove('hidden');
        updateDetectedSymbols();
    } else {
        if (customPanel) customPanel.classList.add('hidden');
        renderSimulatorScenario(scenarioKey);
    }
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
    const symbolInput = document.getElementById('custom-symbol-input');

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
}

function renderCustomSlicing() {
    updateDetectedSymbols();
    executeCustomSlice();
}

// 3. Persistent History & Telemetry Ledger
function initHistoryLedger() {
    renderHistoryLedger();
}

function renderHistoryLedger() {
    const ledgerTable = document.getElementById('history-ledger-body');
    const metrics = SlicingHistoryStore.getMetrics();
    const history = SlicingHistoryStore.getHistory();

    // Update cumulative summary cards
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

// 4. Dynamic Token ROI Financial Calculator
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

        // 9,500 tokens saved average per slice, 21 work days
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

// 5. 31 Supported AI Coding Clients Grid
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

// 6. Clipboard Utility
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
