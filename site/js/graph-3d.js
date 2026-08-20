// 🪐 DAGR 3D WebGL AST Dependency & Pruning Orbit Graph (Three.js)

class Graph3DVisualizer {
    constructor(containerId, tooltipId) {
        this.container = document.getElementById(containerId);
        this.tooltip = document.getElementById(tooltipId);
        if (!this.container || typeof THREE === 'undefined') return;

        this.scene = new THREE.Scene();
        this.camera = new THREE.PerspectiveCamera(45, this.container.clientWidth / 420, 0.1, 1000);
        this.camera.position.set(0, 30, 95);

        this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        this.renderer.setSize(this.container.clientWidth, 420);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        this.container.appendChild(this.renderer.domElement);

        this.nodeMeshes = [];
        this.lines = [];
        this.hoveredObject = null;

        this.isMouseDown = false;
        this.prevMouseX = 0;
        this.prevMouseY = 0;
        this.rotX = 0.2;
        this.rotY = 0;

        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();

        this.initLights();
        this.initEvents();
        this.animate();
    }

    initLights() {
        const ambient = new THREE.AmbientLight(0xffffff, 0.7);
        this.scene.add(ambient);

        const pointLight = new THREE.PointLight(0x10b981, 2, 200);
        pointLight.position.set(0, 10, 40);
        this.scene.add(pointLight);

        const cyanLight = new THREE.PointLight(0x06b6d4, 1.5, 150);
        cyanLight.position.set(-30, -20, 20);
        this.scene.add(cyanLight);
    }

    initEvents() {
        const dom = this.renderer.domElement;

        dom.addEventListener('mousedown', (e) => {
            this.isMouseDown = true;
            this.prevMouseX = e.clientX;
            this.prevMouseY = e.clientY;
        });

        window.addEventListener('mouseup', () => {
            this.isMouseDown = false;
        });

        dom.addEventListener('mousemove', (e) => {
            const rect = dom.getBoundingClientRect();
            this.mouse.x = ((e.clientX - rect.left) / dom.clientWidth) * 2 - 1;
            this.mouse.y = -((e.clientY - rect.top) / dom.clientHeight) * 2 + 1;

            if (this.isMouseDown) {
                const deltaX = e.clientX - this.prevMouseX;
                const deltaY = e.clientY - this.prevMouseY;
                this.rotY += deltaX * 0.008;
                this.rotX += deltaY * 0.008;
                this.prevMouseX = e.clientX;
                this.prevMouseY = e.clientY;
            }

            this.checkHover(e.clientX, e.clientY);
        });

        dom.addEventListener('wheel', (e) => {
            e.preventDefault();
            this.camera.position.z = Math.max(30, Math.min(180, this.camera.position.z + e.deltaY * 0.08));
        });

        window.addEventListener('resize', () => {
            if (!this.container) return;
            const width = this.container.clientWidth;
            this.camera.aspect = width / 420;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(width, 420);
        });
    }

    checkHover(clientX, clientY) {
        this.raycaster.setFromCamera(this.mouse, this.camera);
        const intersects = this.raycaster.intersectObjects(this.nodeMeshes);

        if (intersects.length > 0) {
            const hit = intersects[0].object;
            if (this.hoveredObject !== hit) {
                this.hoveredObject = hit;
                this.showTooltip(hit.userData, clientX, clientY);
            }
        } else {
            if (this.hoveredObject) {
                this.hoveredObject = null;
                this.hideTooltip();
            }
        }
    }

    showTooltip(data, clientX, clientY) {
        if (!this.tooltip || !data) return;

        let badgeHtml = '';
        if (data.type === 'target') {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/30">🟢 3D Target Core</span>';
        } else if (data.type === 'contract') {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-400 font-bold border border-cyan-500/30">🔵 Hoisted Satellite</span>';
        } else {
            badgeHtml = '<span class="px-2 py-0.5 rounded bg-red-500/20 text-red-400 font-bold border border-red-500/30">🔴 Pruned Orbit (-95%)</span>';
        }

        this.tooltip.innerHTML = `
            <div class="space-y-1.5 font-mono text-xs max-w-xs">
                <div class="flex items-center justify-between">
                    <span class="font-bold text-white">${data.label}</span>
                    ${badgeHtml}
                </div>
                <p class="text-zinc-300 text-[11px] leading-relaxed">${data.description}</p>
                <div class="text-[10px] text-zinc-500 pt-1 border-t border-white/10 flex justify-between">
                    <span>Token Footprint: <strong class="text-white">${data.tokens} tokens</strong></span>
                    <span>Status: <strong class="${data.type === 'pruned' ? 'text-red-400' : 'text-emerald-400'}">${data.status}</strong></span>
                </div>
            </div>
        `;

        this.tooltip.classList.remove('hidden');
        const containerRect = this.container.getBoundingClientRect();
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
        // Clear previous meshes
        this.nodeMeshes.forEach(m => this.scene.remove(m));
        this.lines.forEach(l => this.scene.remove(l));
        this.nodeMeshes = [];
        this.lines = [];

        // 1. Context Boundary 3D Wireframe Sphere (Emerald)
        const boundaryGeo = new THREE.IcosahedronGeometry(28, 2);
        const boundaryMat = new THREE.MeshBasicMaterial({
            color: 0x10b981,
            wireframe: true,
            transparent: true,
            opacity: 0.18
        });
        const boundarySphere = new THREE.Mesh(boundaryGeo, boundaryMat);
        this.scene.add(boundarySphere);
        this.nodeMeshes.push(boundarySphere);

        // 2. Center Target Function Sphere (Emerald Core)
        const targetGeo = new THREE.SphereGeometry(4.5, 32, 32);
        const targetMat = new THREE.MeshStandardMaterial({
            color: 0x10b981,
            emissive: 0x059669,
            roughness: 0.2,
            metalness: 0.8
        });
        const targetMesh = new THREE.Mesh(targetGeo, targetMat);
        targetMesh.position.set(0, 0, 0);
        targetMesh.userData = {
            label: targetName || 'TargetFunction',
            type: 'target',
            status: 'KEPT IN 3D CONTEXT',
            tokens: 180,
            description: '🎯 Target Function: Central node isolated by AST traversal.'
        };
        this.scene.add(targetMesh);
        this.nodeMeshes.push(targetMesh);

        // 3. Hoisted Contract Satellites (Cyan Spheres)
        const contractList = contracts.length > 0 ? contracts : ['PayloadContract', 'ReceiptType'];
        contractList.forEach((c, idx) => {
            const angle = (idx / contractList.length) * Math.PI * 2;
            const dist = 16;
            const x = Math.cos(angle) * dist;
            const y = (idx % 2 === 0 ? 6 : -6);
            const z = Math.sin(angle) * dist;

            const geo = new THREE.SphereGeometry(2.8, 24, 24);
            const mat = new THREE.MeshStandardMaterial({
                color: 0x06b6d4,
                emissive: 0x0891b2,
                roughness: 0.3,
                metalness: 0.7
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(x, y, z);
            mesh.userData = {
                label: c,
                type: 'contract',
                status: 'HOISTED CONTRACT',
                tokens: 45,
                description: '🏗️ Upstream Type Contract: Kept inside 3D boundary to prevent LLM hallucinations.'
            };
            this.scene.add(mesh);
            this.nodeMeshes.push(mesh);

            // Connect line to target
            const lineGeo = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3(0, 0, 0),
                new THREE.Vector3(x, y, z)
            ]);
            const lineMat = new THREE.LineBasicMaterial({ color: 0x06b6d4, transparent: true, opacity: 0.6 });
            const line = new THREE.Line(lineGeo, lineMat);
            this.scene.add(line);
            this.lines.push(line);
        });

        // 4. Pruned Outer Satellites (Red Faded Spheres outside boundary)
        const defaultPruned = ['NotificationService', 'DatabasePool', 'RefundWebhook', 'TaxCalculator', 'AuditLogger', 'ExportScript'];
        const prunedList = pruned.length > 0 ? pruned : defaultPruned;

        prunedList.forEach((p, idx) => {
            const phi = Math.acos(-1 + (2 * idx) / prunedList.length);
            const theta = Math.sqrt(prunedList.length * Math.PI) * phi;
            const dist = 42 + (idx % 3) * 6;

            const x = dist * Math.cos(theta) * Math.sin(phi);
            const y = dist * Math.sin(theta) * Math.sin(phi);
            const z = dist * Math.cos(phi);

            const geo = new THREE.SphereGeometry(2.0, 16, 16);
            const mat = new THREE.MeshStandardMaterial({
                color: 0xef4444,
                transparent: true,
                opacity: 0.45,
                roughness: 0.8
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(x, y, z);
            mesh.userData = {
                label: p,
                type: 'pruned',
                status: 'PRUNED FROM 3D CONTEXT',
                tokens: 1250,
                description: '✂️ Pruned Node: Positioned outside the prompt sphere (-95% tokens cut).'
            };
            this.scene.add(mesh);
            this.nodeMeshes.push(mesh);
        });
    }

    animate() {
        requestAnimationFrame(() => this.animate());

        // Gentle idle orbit rotation
        if (!this.isMouseDown) {
            this.rotY += 0.003;
        }

        const radius = this.camera.position.z;
        this.camera.position.x = radius * Math.sin(this.rotY) * Math.cos(this.rotX);
        this.camera.position.y = radius * Math.sin(this.rotX);
        this.camera.position.z = radius * Math.cos(this.rotY) * Math.cos(this.rotX);
        this.camera.lookAt(0, 0, 0);

        this.renderer.render(this.scene, this.camera);
    }
}

// Global 3D Visualizer Singleton
let global3DVisualizer = null;
let activeGraphMode = '2d'; // '2d' or '3d'

function init3DVisualizer() {
    if (!document.getElementById('graph3dContainer')) return;
    global3DVisualizer = new Graph3DVisualizer('graph3dContainer', 'graphTooltip');
    update3DGraphForScenario();
}

function switchGraphMode(mode) {
    activeGraphMode = mode;
    const canvas2d = document.getElementById('astGraphCanvas');
    const container3d = document.getElementById('graph3dContainer');
    const btn2d = document.getElementById('graph-view-btn-2d');
    const btn3d = document.getElementById('graph-view-btn-3d');

    if (mode === '3d') {
        if (canvas2d) canvas2d.classList.add('hidden');
        if (container3d) container3d.classList.remove('hidden');
        if (btn3d) btn3d.className = 'px-3 py-1 rounded-lg bg-cyan-500/20 text-cyan-400 font-bold border border-cyan-500/40 text-xs font-mono transition-all';
        if (btn2d) btn2d.className = 'px-3 py-1 rounded-lg text-zinc-400 hover:text-white border border-transparent text-xs font-mono transition-all';
        
        if (!global3DVisualizer) {
            init3DVisualizer();
        } else {
            update3DGraphForScenario();
        }
    } else {
        if (canvas2d) canvas2d.classList.remove('hidden');
        if (container3d) container3d.classList.add('hidden');
        if (btn2d) btn2d.className = 'px-3 py-1 rounded-lg bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/40 text-xs font-mono transition-all';
        if (btn3d) btn3d.className = 'px-3 py-1 rounded-lg text-zinc-400 hover:text-white border border-transparent text-xs font-mono transition-all';
        
        if (globalGraphVisualizer) {
            globalGraphVisualizer.resize();
        }
    }
}

function update3DGraphForScenario() {
    if (!global3DVisualizer) return;
    if (activeScenario === 'typescript') {
        global3DVisualizer.loadScenario('processPayment', ['PaymentPayload', 'PaymentReceipt'], ['StripeClient', 'NotificationService', 'DatabasePool', 'InvoiceGenerator', 'AuditLogger', 'CurrencyConverter']);
    } else if (activeScenario === 'python') {
        global3DVisualizer.loadScenario('verify_token', ['AuthToken', 'TokenValidationResult'], ['RateLimiter', 'UserRecord', 'get_db', 'send_otp_email', 'hashlib']);
    } else if (activeScenario === 'rust') {
        global3DVisualizer.loadScenario('open_database', ['DbConfig', 'StorageMetrics'], ['RawPool', 'WalSyncWorker', 'SchemaMigration', 'CompressionEngine', 'FtsIndexer']);
    } else {
        const target = document.getElementById('custom-symbol-input')?.value || 'customFunction';
        global3DVisualizer.loadScenario(target, ['TargetContract', 'RequiredType'], ['UnrelatedHelperA', 'UnrelatedHelperB', 'DatabaseClient', 'TaxModule']);
    }
}
