// 🕸️ DAGR Layman-Friendly AST Dependency & Pruning Graph Visualizer

class GraphVisualizer {
    constructor(canvasId, tooltipId) {
        this.canvas = document.getElementById(canvasId);
        this.tooltip = document.getElementById(tooltipId);
        if (!this.canvas) return;
        this.ctx = this.canvas.getContext('2d');

        this.nodes = [];
        this.links = [];
        this.hoveredNode = null;
        this.selectedNode = null;

        this.panX = 0;
        this.panY = 0;
        this.zoom = 1;
        this.isDragging = false;
        this.dragStartX = 0;
        this.dragStartY = 0;

        this.initEvents();
        this.resize();
        window.addEventListener('resize', () => this.resize());
    }

    resize() {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = rect.width * window.devicePixelRatio;
        this.canvas.height = 420 * window.devicePixelRatio;
        this.canvas.style.width = `${rect.width}px`;
        this.canvas.style.height = `420px`;
        this.draw();
    }

    initEvents() {
        if (!this.canvas) return;

        this.canvas.addEventListener('mousemove', (e) => {
            const rect = this.canvas.getBoundingClientRect();
            const mouseX = (e.clientX - rect.left) * window.devicePixelRatio;
            const mouseY = (e.clientY - rect.top) * window.devicePixelRatio;

            if (this.isDragging) {
                this.panX += (mouseX - this.dragStartX);
                this.panY += (mouseY - this.dragStartY);
                this.dragStartX = mouseX;
                this.dragStartY = mouseY;
                this.draw();
                return;
            }

            // Find hovered node
            const transformedX = (mouseX - this.panX) / this.zoom;
            const transformedY = (mouseY - this.panY) / this.zoom;

            let hit = null;
            for (const n of this.nodes) {
                const dist = Math.hypot(n.x - transformedX, n.y - transformedY);
                if (dist <= n.radius * 1.5) {
                    hit = n;
                    break;
                }
            }

            if (hit !== this.hoveredNode) {
                this.hoveredNode = hit;
                this.draw();
                this.updateTooltip(e.clientX, e.clientY);
            }
        });

        this.canvas.addEventListener('mousedown', (e) => {
            const rect = this.canvas.getBoundingClientRect();
            this.isDragging = true;
            this.dragStartX = (e.clientX - rect.left) * window.devicePixelRatio;
            this.dragStartY = (e.clientY - rect.top) * window.devicePixelRatio;
            if (this.hoveredNode) {
                this.selectedNode = this.hoveredNode;
                this.updateTooltip(e.clientX, e.clientY);
            }
        });

        window.addEventListener('mouseup', () => {
            this.isDragging = false;
        });

        this.canvas.addEventListener('mouseleave', () => {
            this.hoveredNode = null;
            this.hideTooltip();
            this.draw();
        });
    }

    updateTooltip(clientX, clientY) {
        if (!this.tooltip) return;
        const node = this.hoveredNode || this.selectedNode;
        if (!node) {
            this.hideTooltip();
            return;
        }

        let badgeHtml = '';
        if (node.type === 'target') {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/30">🟢 Kept in LLM Prompt</span>';
        } else if (node.type === 'contract') {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-400 font-bold border border-cyan-500/30">🔵 Hoisted Type Contract</span>';
        } else {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 font-bold border border-red-500/30">🔴 Pruned From Context (-95%)</span>';
        }

        this.tooltip.innerHTML = `
            <div class="space-y-1.5 font-mono text-xs max-w-xs">
                <div class="flex items-center justify-between">
                    <span class="font-bold text-white">${node.label}</span>
                    ${badgeHtml}
                </div>
                <p class="text-zinc-300 text-[11px] leading-relaxed">${node.description}</p>
                <div class="text-[10px] text-zinc-500 pt-1 border-t border-white/10 flex justify-between">
                    <span>Token Impact: <strong class="text-white">${node.tokens} tokens</strong></span>
                    <span>Status: <strong class="${node.type === 'pruned' ? 'text-red-400' : 'text-emerald-400'}">${node.status}</strong></span>
                </div>
            </div>
        `;

        this.tooltip.classList.remove('hidden');
        const containerRect = this.canvas.parentElement.getBoundingClientRect();
        let left = clientX - containerRect.left + 15;
        let top = clientY - containerRect.top + 15;

        if (left + 280 > containerRect.width) left = left - 300;
        if (top + 120 > 420) top = top - 130;

        this.tooltip.style.left = `${Math.max(10, left)}px`;
        this.tooltip.style.top = `${Math.max(10, top)}px`;
    }

    hideTooltip() {
        if (this.tooltip) this.tooltip.classList.add('hidden');
    }

    loadScenario(targetName, contracts = [], pruned = []) {
        this.nodes = [];
        this.links = [];

        const centerX = (this.canvas.width / 2);
        const centerY = (this.canvas.height / 2);

        // 1. Target Node (Green Center)
        const targetNode = {
            id: 'target',
            label: targetName || 'TargetFunction',
            type: 'target',
            x: centerX,
            y: centerY,
            radius: 26 * window.devicePixelRatio,
            color: '#10b981',
            status: 'KEPT IN CONTEXT',
            tokens: 180,
            description: '🎯 Target Function: The exact symbol your prompt needs to inspect or edit.'
        };
        this.nodes.push(targetNode);

        // 2. Hoisted Contracts (Cyan Nodes surrounding target)
        const contractList = contracts.length > 0 ? contracts : ['PayloadContract', 'ReceiptType'];
        contractList.forEach((c, idx) => {
            const angle = (idx / contractList.length) * Math.PI * 1.2 - Math.PI * 0.6;
            const dist = 110 * window.devicePixelRatio;
            const node = {
                id: `contract_${idx}`,
                label: c,
                type: 'contract',
                x: centerX + Math.cos(angle) * dist,
                y: centerY - Math.abs(Math.sin(angle)) * dist - 30 * window.devicePixelRatio,
                radius: 20 * window.devicePixelRatio,
                color: '#06b6d4',
                status: 'HOISTED CONTRACT',
                tokens: 45,
                description: '🏗️ Upstream Type Contract: Hoisted to preserve parameter types so the AI won\'t hallucinate invalid signatures.'
            };
            this.nodes.push(node);
            this.links.push({ from: targetNode, to: node, type: 'contract' });
        });

        // 3. Pruned Background Nodes (Faded Red Nodes outside perimeter)
        const defaultPruned = ['NotificationService', 'DatabasePool', 'RefundWebhook', 'TaxCalculator', 'AuditLogger', 'ExportScript', 'MetricsWorker'];
        const prunedList = pruned.length > 0 ? pruned : defaultPruned;

        prunedList.forEach((p, idx) => {
            const angle = (idx / prunedList.length) * Math.PI * 2;
            const dist = (200 + (idx % 2) * 35) * window.devicePixelRatio;
            const node = {
                id: `pruned_${idx}`,
                label: p,
                type: 'pruned',
                x: centerX + Math.cos(angle) * dist,
                y: centerY + Math.sin(angle) * dist,
                radius: 16 * window.devicePixelRatio,
                color: 'rgba(239, 68, 68, 0.45)',
                status: 'PRUNED FROM CONTEXT',
                tokens: 1450,
                description: '✂️ Irrelevant Monolithic Code: Omitted from the LLM prompt to eliminate token bloat and avoid lost-in-the-middle confusion.'
            };
            this.nodes.push(node);
            this.links.push({ from: targetNode, to: node, type: 'pruned' });
        });

        this.panX = 0;
        this.panY = 0;
        this.draw();
    }

    draw() {
        if (!this.ctx) return;
        const ctx = this.ctx;
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        ctx.save();
        ctx.translate(this.panX, this.panY);
        ctx.scale(this.zoom, this.zoom);

        const centerX = this.canvas.width / 2;
        const centerY = this.canvas.height / 2;

        // Draw LLM Context Safe Perimeter Ring (Emerald Dashed Line)
        ctx.beginPath();
        ctx.arc(centerX, centerY, 155 * window.devicePixelRatio, 0, Math.PI * 2);
        ctx.setLineDash([8, 8]);
        ctx.strokeStyle = 'rgba(16, 185, 129, 0.3)';
        ctx.lineWidth = 2 * window.devicePixelRatio;
        ctx.stroke();
        ctx.setLineDash([]);

        // Label for Context Boundary
        ctx.font = `${10 * window.devicePixelRatio}px JetBrains Mono, monospace`;
        ctx.fillStyle = 'rgba(16, 185, 129, 0.7)';
        ctx.fillText('⚡ DAGR LLM PROMPT BOUNDARY (95% TOKENS SAVED OUTSIDE)', centerX - 140 * window.devicePixelRatio, centerY - 165 * window.devicePixelRatio);

        // Draw Links
        for (const l of this.links) {
            ctx.beginPath();
            ctx.moveTo(l.from.x, l.from.y);
            ctx.lineTo(l.to.x, l.to.y);

            if (l.type === 'contract') {
                ctx.strokeStyle = 'rgba(6, 182, 212, 0.6)';
                ctx.lineWidth = 2 * window.devicePixelRatio;
                ctx.stroke();
            } else {
                ctx.setLineDash([4, 4]);
                ctx.strokeStyle = 'rgba(239, 68, 68, 0.2)';
                ctx.lineWidth = 1 * window.devicePixelRatio;
                ctx.stroke();
                ctx.setLineDash([]);
            }
        }

        // Draw Nodes
        for (const n of this.nodes) {
            const isHovered = this.hoveredNode === n || this.selectedNode === n;

            ctx.beginPath();
            ctx.arc(n.x, n.y, n.radius + (isHovered ? 4 : 0), 0, Math.PI * 2);

            if (n.type === 'target') {
                ctx.fillStyle = isHovered ? '#34d399' : '#10b981';
                ctx.shadowColor = 'rgba(16, 185, 129, 0.6)';
                ctx.shadowBlur = isHovered ? 25 : 15;
            } else if (n.type === 'contract') {
                ctx.fillStyle = isHovered ? '#22d3ee' : '#06b6d4';
                ctx.shadowColor = 'rgba(6, 182, 212, 0.5)';
                ctx.shadowBlur = isHovered ? 20 : 10;
            } else {
                ctx.fillStyle = isHovered ? 'rgba(239, 68, 68, 0.8)' : 'rgba(239, 68, 68, 0.35)';
                ctx.shadowColor = 'rgba(0, 0, 0, 0)';
                ctx.shadowBlur = 0;
            }

            ctx.fill();
            ctx.shadowBlur = 0;

            ctx.strokeStyle = isHovered ? '#ffffff' : 'rgba(255, 255, 255, 0.2)';
            ctx.lineWidth = 1.5 * window.devicePixelRatio;
            ctx.stroke();

            // Node Text Label
            ctx.font = `${11 * window.devicePixelRatio}px JetBrains Mono, monospace`;
            ctx.fillStyle = isHovered ? '#ffffff' : (n.type === 'pruned' ? 'rgba(244, 114, 182, 0.6)' : '#f3f4f6');
            ctx.textAlign = 'center';
            ctx.fillText(n.label, n.x, n.y + n.radius + 14 * window.devicePixelRatio);
        }

        ctx.restore();
    }
}

// Global visualizer singleton
let globalGraphVisualizer = null;

function initGraphVisualizer() {
    if (!document.getElementById('astGraphCanvas')) return;
    globalGraphVisualizer = new GraphVisualizer('astGraphCanvas', 'graphTooltip');
    updateGraphForActiveScenario();
}

function updateGraphForActiveScenario() {
    if (!globalGraphVisualizer) return;
    
    if (activeScenario === 'typescript') {
        globalGraphVisualizer.loadScenario('processPayment', ['PaymentPayload', 'PaymentReceipt'], ['StripeClient', 'NotificationService', 'DatabasePool', 'InvoiceGenerator', 'AuditLogger', 'CurrencyConverter']);
    } else if (activeScenario === 'python') {
        globalGraphVisualizer.loadScenario('verify_token', ['AuthToken', 'TokenValidationResult'], ['RateLimiter', 'UserRecord', 'get_db', 'send_otp_email', 'hashlib']);
    } else if (activeScenario === 'rust') {
        globalGraphVisualizer.loadScenario('open_database', ['DbConfig', 'StorageMetrics'], ['RawPool', 'WalSyncWorker', 'SchemaMigration', 'CompressionEngine', 'FtsIndexer']);
    } else {
        const target = document.getElementById('custom-symbol-input')?.value || 'customFunction';
        globalGraphVisualizer.loadScenario(target, ['TargetContract', 'RequiredType'], ['UnrelatedHelper1', 'UnrelatedHelper2', 'UnrelatedHelper3', 'DeadCodeBlock']);
    }
}
