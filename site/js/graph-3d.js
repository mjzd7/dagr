// 🪐 DAGR 3D WebGL AST Dependency & Pruning Orbit Graph with 3D Billboard Labels & Zero-Hang Engine

class Graph3DVisualizer {
    constructor(containerId, tooltipId) {
        this.container = document.getElementById(containerId);
        this.tooltip = document.getElementById(tooltipId);
        if (!this.container || typeof THREE === 'undefined') return;

        this.scene = new THREE.Scene();
        this.camera = new THREE.PerspectiveCamera(45, this.container.clientWidth / (this.container.clientHeight || 420), 0.1, 1000);
        this.camera.position.set(0, 25, 90);

        this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        this.renderer.setSize(this.container.clientWidth, this.container.clientHeight || 420);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        this.container.innerHTML = '';
        this.container.appendChild(this.renderer.domElement);

        this.nodeMeshes = [];
        this.labelSprites = [];
        this.lines = [];
        this.hoveredObject = null;

        this.isMouseDown = false;
        this.prevMouseX = 0;
        this.prevMouseY = 0;
        this.rotX = 0.25;
        this.rotY = 0;
        this.autoOrbit = true;

        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();
        this.animFrameId = null;
        this.isDestroyed = false;

        this.initLights();
        this.initEvents();
        this.startLoop();
    }

    initLights() {
        const ambient = new THREE.AmbientLight(0xffffff, 0.75);
        this.scene.add(ambient);

        const emeraldPoint = new THREE.PointLight(0x10b981, 2.5, 200);
        emeraldPoint.position.set(0, 15, 30);
        this.scene.add(emeraldPoint);

        const cyanPoint = new THREE.PointLight(0x06b6d4, 2.0, 180);
        cyanPoint.position.set(-25, -20, 25);
        this.scene.add(cyanPoint);

        const redPoint = new THREE.PointLight(0xef4444, 1.2, 160);
        redPoint.position.set(35, 20, -30);
        this.scene.add(redPoint);
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
                
                this.rotY += deltaX * 0.007;
                // Clamp pitch between -1.35 and 1.35 to prevent gimbal lock and NaN hangs
                this.rotX = Math.max(-1.35, Math.min(1.35, this.rotX + deltaY * 0.007));
                
                this.prevMouseX = e.clientX;
                this.prevMouseY = e.clientY;
            }

            this.checkHover(e.clientX, e.clientY);
        });

        dom.addEventListener('wheel', (e) => {
            e.preventDefault();
            this.camera.position.z = Math.max(25, Math.min(160, this.camera.position.z + e.deltaY * 0.07));
        });

        window.addEventListener('resize', () => this.resize());
    }

    resize() {
        if (!this.container || !this.renderer || !this.camera) return;
        const width = this.container.clientWidth || 800;
        const height = this.container.clientHeight || 420;
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(width, height);
    }

    createLabelSprite(text, type) {
        const canvas = document.createElement('canvas');
        canvas.width = 380;
        canvas.height = 80;
        const ctx = canvas.getContext('2d');

        let bgColor = 'rgba(16, 185, 129, 0.85)';
        let borderColor = '#34d399';
        let prefix = '🎯 [TARGET] ';
        let textColor = '#ffffff';

        if (type === 'contract') {
            bgColor = 'rgba(6, 182, 212, 0.85)';
            borderColor = '#22d3ee';
            prefix = '🏗️ [CONTRACT] ';
        } else if (type === 'pruned') {
            bgColor = 'rgba(239, 68, 68, 0.75)';
            borderColor = '#f87171';
            prefix = '✂️ [PRUNED] ';
        }

        // Draw pill container
        ctx.fillStyle = bgColor;
        ctx.strokeStyle = borderColor;
        ctx.lineWidth = 4;
        
        ctx.beginPath();
        ctx.roundRect(10, 10, 360, 60, 14);
        ctx.fill();
        ctx.stroke();

        // Draw text
        ctx.fillStyle = textColor;
        ctx.font = 'bold 24px JetBrains Mono, monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(prefix + text, 190, 40);

        const texture = new THREE.CanvasTexture(canvas);
        texture.minFilter = THREE.LinearFilter;
        const spriteMat = new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false });
        const sprite = new THREE.Sprite(spriteMat);
        sprite.scale.set(15, 3.2, 1);
        return sprite;
    }

    loadScenario(targetName, contracts = [], pruned = []) {
        // Safe disposal of existing meshes and geometries
        this.cleanupScene();

        // 1. LLM Context Boundary 3D Wireframe Icosahedron (Emerald Sphere)
        const boundaryGeo = new THREE.IcosahedronGeometry(28, 2);
        const boundaryMat = new THREE.MeshBasicMaterial({
            color: 0x10b981,
            wireframe: true,
            transparent: true,
            opacity: 0.22
        });
        const boundarySphere = new THREE.Mesh(boundaryGeo, boundaryMat);
        this.scene.add(boundarySphere);
        this.nodeMeshes.push(boundarySphere);

        // 2. Center Target Function Sphere (Green Core)
        const targetGeo = new THREE.SphereGeometry(4.5, 32, 32);
        const targetMat = new THREE.MeshStandardMaterial({
            color: 0x10b981,
            emissive: 0x059669,
            roughness: 0.15,
            metalness: 0.85
        });
        const targetMesh = new THREE.Mesh(targetGeo, targetMat);
        targetMesh.position.set(0, 0, 0);
        targetMesh.userData = {
            label: targetName || 'TargetFunction',
            type: 'target',
            status: 'KEPT IN 3D CONTEXT',
            tokens: 180,
            description: '🎯 Target Function: Central execution symbol requested by prompt.'
        };
        this.scene.add(targetMesh);
        this.nodeMeshes.push(targetMesh);

        // Add 3D Billboard Label for Target
        const targetLabel = this.createLabelSprite(targetName || 'TargetFunction', 'target');
        targetLabel.position.set(0, 7.5, 0);
        this.scene.add(targetLabel);
        this.labelSprites.push(targetLabel);

        // 3. Hoisted Upstream Contract Satellites (Cyan Spheres)
        const contractList = contracts.length > 0 ? contracts : ['PaymentPayload', 'PaymentReceipt'];
        contractList.forEach((c, idx) => {
            const angle = (idx / contractList.length) * Math.PI * 2;
            const dist = 16.5;
            const x = Math.cos(angle) * dist;
            const y = (idx % 2 === 0 ? 5.5 : -5.5);
            const z = Math.sin(angle) * dist;

            const geo = new THREE.SphereGeometry(2.8, 24, 24);
            const mat = new THREE.MeshStandardMaterial({
                color: 0x06b6d4,
                emissive: 0x0891b2,
                roughness: 0.25,
                metalness: 0.75
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(x, y, z);
            mesh.userData = {
                label: c,
                type: 'contract',
                status: 'HOISTED CONTRACT',
                tokens: 45,
                description: '🏗️ Upstream Type Contract: Hoisted inside prompt boundary to prevent LLM hallucinations.'
            };
            this.scene.add(mesh);
            this.nodeMeshes.push(mesh);

            // Add 3D Billboard Label
            const label = this.createLabelSprite(c, 'contract');
            label.position.set(x, y + 5.2, z);
            this.scene.add(label);
            this.labelSprites.push(label);

            // Connect vector tube to target
            const lineGeo = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3(0, 0, 0),
                new THREE.Vector3(x, y, z)
            ]);
            const lineMat = new THREE.LineBasicMaterial({ color: 0x06b6d4, transparent: true, opacity: 0.75, linewidth: 2 });
            const line = new THREE.Line(lineGeo, lineMat);
            this.scene.add(line);
            this.lines.push(line);
        });

        // 4. Pruned Outer Debris (Red Spheres drifting outside boundary)
        const defaultPruned = ['NotificationService', 'DatabasePool', 'RefundWebhook', 'TaxCalculator', 'AuditLogger', 'ExportScript'];
        const prunedList = pruned.length > 0 ? pruned : defaultPruned;

        prunedList.forEach((p, idx) => {
            const phi = Math.acos(-1 + (2 * idx) / prunedList.length);
            const theta = Math.sqrt(prunedList.length * Math.PI) * phi;
            const dist = 42 + (idx % 3) * 6;

            const x = dist * Math.cos(theta) * Math.sin(phi);
            const y = dist * Math.sin(theta) * Math.sin(phi);
            const z = dist * Math.cos(phi);

            const geo = new THREE.SphereGeometry(2.2, 16, 16);
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
                description: '✂️ Pruned Monolith Code: Omitted from LLM prompt (-95% token savings).'
            };
            this.scene.add(mesh);
            this.nodeMeshes.push(mesh);

            // Add 3D Billboard Label for pruned nodes
            const label = this.createLabelSprite(p, 'pruned');
            label.position.set(x, y + 4.2, z);
            this.scene.add(label);
            this.labelSprites.push(label);
        });
    }

    cleanupScene() {
        this.nodeMeshes.forEach(m => {
            if (m.geometry) m.geometry.dispose();
            if (m.material) {
                if (Array.isArray(m.material)) m.material.forEach(mat => mat.dispose());
                else m.material.dispose();
            }
            this.scene.remove(m);
        });
        this.labelSprites.forEach(s => {
            if (s.material.map) s.material.map.dispose();
            if (s.material) s.material.dispose();
            this.scene.remove(s);
        });
        this.lines.forEach(l => {
            if (l.geometry) l.geometry.dispose();
            if (l.material) l.material.dispose();
            this.scene.remove(l);
        });
        this.nodeMeshes = [];
        this.labelSprites = [];
        this.lines = [];
    }

    checkHover(clientX, clientY) {
        this.raycaster.setFromCamera(this.mouse, this.camera);
        const intersects = this.raycaster.intersectObjects(this.nodeMeshes);

        if (intersects.length > 0) {
            const hit = intersects[0].object;
            if (hit.userData && hit.userData.label && this.hoveredObject !== hit) {
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
        if (top + 120 > containerRect.height) top = top - 130;

        this.tooltip.style.left = `${Math.max(10, left)}px`;
        this.tooltip.style.top = `${Math.max(10, top)}px`;
    }

    hideTooltip() {
        if (this.tooltip) this.tooltip.classList.add('hidden');
    }

    resetCamera() {
        this.rotX = 0.25;
        this.rotY = 0;
        this.camera.position.set(0, 25, 90);
    }

    toggleAutoOrbit() {
        this.autoOrbit = !this.autoOrbit;
        return this.autoOrbit;
    }

    startLoop() {
        if (this.animFrameId) cancelAnimationFrame(this.animFrameId);

        const render = () => {
            if (this.isDestroyed) return;
            this.animFrameId = requestAnimationFrame(render);

            // Smooth auto-orbit rotation when user is not actively dragging
            if (!this.isMouseDown && this.autoOrbit) {
                this.rotY += 0.0035;
            }

            const radius = this.camera.position.z;
            this.camera.position.x = radius * Math.sin(this.rotY) * Math.cos(this.rotX);
            this.camera.position.y = radius * Math.sin(this.rotX);
            this.camera.position.z = radius * Math.cos(this.rotY) * Math.cos(this.rotX);
            this.camera.lookAt(0, 0, 0);

            this.renderer.render(this.scene, this.camera);
        };

        render();
    }

    destroy() {
        this.isDestroyed = true;
        if (this.animFrameId) cancelAnimationFrame(this.animFrameId);
        this.cleanupScene();
        if (this.renderer && this.renderer.domElement && this.renderer.domElement.parentElement) {
            this.renderer.domElement.parentElement.removeChild(this.renderer.domElement);
        }
    }
}

// Global 3D Visualizer Singleton
let global3DVisualizer = null;
let activeGraphMode = '2d'; // '2d' or '3d'

function init3DVisualizer() {
    if (!document.getElementById('graph3dContainer')) return;
    if (global3DVisualizer) {
        global3DVisualizer.destroy();
    }
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
        
        init3DVisualizer();
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

// Fullscreen Modal Controls
function openGraphFullscreen() {
    const modal = document.getElementById('graph-fullscreen-modal');
    const modalTarget = document.getElementById('fullscreen-graph-target');
    if (!modal || !modalTarget) return;

    modal.classList.remove('hidden');
    document.body.style.overflow = 'hidden';

    // Move 3D or 2D canvas into fullscreen modal
    const originalWrapper = document.getElementById('graph2dWrapper');
    if (originalWrapper) {
        modalTarget.appendChild(originalWrapper);
        if (global3DVisualizer && activeGraphMode === '3d') {
            setTimeout(() => global3DVisualizer.resize(), 100);
        } else if (globalGraphVisualizer) {
            setTimeout(() => globalGraphVisualizer.resize(), 100);
        }
    }
}

function closeGraphFullscreen() {
    const modal = document.getElementById('graph-fullscreen-modal');
    const originalHost = document.getElementById('graph-original-host');
    const originalWrapper = document.getElementById('graph2dWrapper');

    if (!modal || !originalHost || !originalWrapper) return;

    modal.classList.add('hidden');
    document.body.style.overflow = '';

    originalHost.appendChild(originalWrapper);
    if (global3DVisualizer && activeGraphMode === '3d') {
        setTimeout(() => global3DVisualizer.resize(), 100);
    } else if (globalGraphVisualizer) {
        setTimeout(() => globalGraphVisualizer.resize(), 100);
    }
}
