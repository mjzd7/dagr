// 📊 DAGR Persistent Simulator Slicing Database & Telemetry Ledger

class SlicingHistoryStore {
    static STORAGE_KEY = 'dagr_simulator_telemetry_db';

    static getHistory() {
        try {
            const raw = localStorage.getItem(this.STORAGE_KEY);
            if (!raw) return [];
            return JSON.parse(raw);
        } catch (e) {
            console.error('Failed to load slicing history:', e);
            return [];
        }
    }

    static addRecord(sliceData) {
        const history = this.getHistory();
        const newRecord = {
            id: `slice_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`,
            timestamp: Date.now(),
            dateFormatted: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            language: sliceData.language || 'typescript',
            targetSymbol: sliceData.targetSymbol || 'anonymous',
            rawTokens: sliceData.rawTokens || 0,
            slicedTokens: sliceData.slicedTokens || 0,
            tokensSaved: sliceData.tokensSaved || 0,
            compressionPct: sliceData.compressionPct || '0.0',
            usdSaved: sliceData.usdSaved || '0.000',
            linesPruned: sliceData.linesPruned || 0
        };

        // Keep latest 50 iterations in ledger
        history.unshift(newRecord);
        if (history.length > 50) history.pop();

        try {
            localStorage.setItem(this.STORAGE_KEY, JSON.stringify(history));
        } catch (e) {
            console.warn('localStorage write failed:', e);
        }

        return newRecord;
    }

    static getMetrics() {
        const history = this.getHistory();
        if (history.length === 0) {
            return {
                totalSlices: 0,
                totalTokensSaved: 0,
                totalUsdSaved: '0.00',
                avgCompression: '0.0%'
            };
        }

        const totalTokensSaved = history.reduce((acc, cur) => acc + (cur.tokensSaved || 0), 0);
        const totalUsd = ((totalTokensSaved / 1_000_000) * 3.0).toFixed(2);
        
        const sumPct = history.reduce((acc, cur) => acc + parseFloat(cur.compressionPct || 0), 0);
        const avgCompression = (sumPct / history.length).toFixed(1) + '%';

        return {
            totalSlices: history.length,
            totalTokensSaved,
            totalUsdSaved: `$${totalUsd}`,
            avgCompression
        };
    }

    static clear() {
        localStorage.removeItem(this.STORAGE_KEY);
    }
}
