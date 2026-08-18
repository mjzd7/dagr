// DAGR Interactive Landing Page Engine

document.addEventListener('DOMContentLoaded', () => {
    initSimulator();
    initRoiCalculator();
    initClientsGrid();
    initClipboardButtons();
});

// 1. Interactive AST Slicing Simulator
let activeScenario = 'typescript';

function initSimulator() {
    renderSimulatorScenario(activeScenario);
}

function selectScenario(scenarioKey) {
    activeScenario = scenarioKey;
    ['typescript', 'python', 'rust'].forEach(k => {
        const btn = document.getElementById(`sim-btn-${k}`);
        if (btn) {
            if (k === scenarioKey) {
                btn.className = 'px-4 py-2 rounded-xl bg-white/10 text-white font-semibold text-xs border border-white/20 shadow-sm transition-all';
            } else {
                btn.className = 'px-4 py-2 rounded-xl text-zinc-400 hover:text-white text-xs border border-transparent transition-all';
            }
        }
    });
    renderSimulatorScenario(scenarioKey);
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
}

// 2. Dynamic Token ROI Financial Calculator
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

        // Mathematical formulation:
        // Average token reduction per slice = 9,500 tokens
        // Work days per month = 21
        const tokensSavedPerMonth = teamSize * promptsPerDay * 9500 * 21;
        const usdSavedPerMonth = (tokensSavedPerMonth / 1_000_000) * pricePerM;
        const hoursSavedPerYear = Math.round(teamSize * 2.5 * 12); // ~2.5 hrs saved per dev/mo from avoid dirty git resets

        document.getElementById('calc-tokens-monthly').innerText = `${(tokensSavedPerMonth / 1_000_000).toFixed(1)}M`;
        document.getElementById('calc-usd-monthly').innerText = `$${Math.round(usdSavedPerMonth).toLocaleString()}`;
        document.getElementById('calc-hours-annual').innerText = `${hoursSavedPerYear.toLocaleString()} hrs`;
    }

    teamSlider.addEventListener('input', recalculate);
    promptsSlider.addEventListener('input', recalculate);
    modelSelect.addEventListener('change', recalculate);
    recalculate();
}

// 3. 31 Supported AI Coding Clients Grid
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

// 4. Resilient Clipboard Utility
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
