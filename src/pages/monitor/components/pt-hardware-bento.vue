<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import {
  Cpu,
  Layers,
  Thermometer,
  Zap,
  Activity,
  SlidersHorizontal,
  ChevronRight,
  ShieldCheck,
} from '@lucide/vue';
import { formatBytes } from '@/lib/utils/format';
import { ProcessService, type TopProcessInfo } from '@/lib/services/api-services';
import { useMonitorStore } from '@/stores/monitor-store';
import { useMemoryCleanerStore } from '@/stores/memory-cleaner-store';
import PtMemoryDrawer from './pt-memory-drawer.vue';

const monitor = useMonitorStore();
const memory = useMemoryCleanerStore();

const memoryDrawerOpen = ref(false);
const topProcesses = ref<TopProcessInfo[]>([]);
const loadingProcesses = ref(false);

// Live Sparkline History (60 samples)
const cpuHistory = ref<number[]>(Array(40).fill(15));
const ramHistory = ref<number[]>(Array(40).fill(50));
const gpuHistory = ref<number[]>(Array(40).fill(10));

let historyTimer: number | null = null;

const snap = computed(() => monitor.snapshot);
const memStats = computed(() => memory.stats);

const cpuPercent = computed(() => snap.value?.cpu ?? 0);
const ramPercent = computed(() => {
  const fromMem = memStats.value?.physicalLoadPercent;
  return fromMem != null && Number.isFinite(fromMem)
    ? fromMem
    : (snap.value?.memoryPercent ?? 0);
});
const gpuPercent = computed(() => snap.value?.gpuUtilization ?? 0);
const gpuTemp = computed(() => snap.value?.gpuTemperature);

// Sparkline SVG path generator
function makeSvgPath(points: number[], width = 280, height = 55): string {
  if (points.length === 0) return '';
  const step = width / (points.length - 1);
  return points
    .map((val, idx) => {
      const clamped = Math.min(100, Math.max(0, val));
      const x = idx * step;
      const y = height - (clamped / 100) * height;
      return `${idx === 0 ? 'M' : 'L'} ${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(' ');
}

async function refreshTopProcesses() {
  try {
    const list = await ProcessService.getTopProcesses(3);
    topProcesses.value = list;
  } catch (err) {
    // Silently continue
  }
}

onMounted(() => {
  void refreshTopProcesses();
  historyTimer = window.setInterval(() => {
    cpuHistory.value.push(cpuPercent.value);
    if (cpuHistory.value.length > 40) cpuHistory.value.shift();

    ramHistory.value.push(ramPercent.value);
    if (ramHistory.value.length > 40) ramHistory.value.shift();

    gpuHistory.value.push(gpuPercent.value);
    if (gpuHistory.value.length > 40) gpuHistory.value.shift();
  }, 1000);
});

onUnmounted(() => {
  if (historyTimer) window.clearInterval(historyTimer);
});
</script>

<template>
  <div class="hardware-grid">
    <!-- CPU BENTO CARD -->
    <div class="bento-card bento-card--cpu">
      <div class="card-bg-sparkline">
        <svg viewBox="0 0 280 55" preserveAspectRatio="none" class="sparkline-svg">
          <defs>
            <linearGradient id="cpuGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#06b6d4" stop-opacity="0.25" />
              <stop offset="100%" stop-color="#06b6d4" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path :d="makeSvgPath(cpuHistory) + ' L 280,55 L 0,55 Z'" fill="url(#cpuGrad)" />
          <path :d="makeSvgPath(cpuHistory)" fill="none" stroke="#06b6d4" stroke-width="1.75" />
        </svg>
      </div>

      <div class="card-header">
        <div class="badge-title">
          <Cpu :size="16" class="text-cyan" />
          <span>CPU TELEMETRY</span>
        </div>
        <span class="live-dot" />
      </div>

      <div class="card-hero">
        <div class="hero-left">
          <span class="hero-val font-mono">{{ cpuPercent.toFixed(1) }}%</span>
          <span class="hero-label">Core Load</span>
        </div>
        <div class="hero-right font-mono">
          <div class="spec-row">
            <span class="spec-label">Xung nhịp:</span>
            <span class="spec-val">3.20 GHz</span>
          </div>
          <div class="spec-row">
            <span class="spec-label">Luồng/Nhân:</span>
            <span class="spec-val">16T / 8C</span>
          </div>
        </div>
      </div>

      <div class="card-footer-info font-mono">
        <div class="sub-stat">
          <Activity :size="13" class="text-cyan" />
          <span>228 Processes</span>
        </div>
        <div class="sub-stat">
          <Thermometer :size="13" class="text-cyan" />
          <span>~52°C</span>
        </div>
      </div>
    </div>

    <!-- RAM BENTO CARD -->
    <div class="bento-card bento-card--ram">
      <div class="card-bg-sparkline">
        <svg viewBox="0 0 280 55" preserveAspectRatio="none" class="sparkline-svg">
          <defs>
            <linearGradient id="ramGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#8b5cf6" stop-opacity="0.25" />
              <stop offset="100%" stop-color="#8b5cf6" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path :d="makeSvgPath(ramHistory) + ' L 280,55 L 0,55 Z'" fill="url(#ramGrad)" />
          <path :d="makeSvgPath(ramHistory)" fill="none" stroke="#8b5cf6" stroke-width="1.75" />
        </svg>
      </div>

      <div class="card-header">
        <div class="badge-title">
          <Layers :size="16" class="text-purple" />
          <span>RAM ALLOCATION</span>
        </div>
        <button
          type="button"
          class="stage-inspect-btn"
          title="Mở bảng phân tích bộ nhớ & tiến trình"
          @click="memoryDrawerOpen = true"
        >
          <SlidersHorizontal :size="13" />
          <span>Phân tích</span>
          <ChevronRight :size="13" />
        </button>
      </div>

      <div class="card-hero">
        <div class="hero-left">
          <span class="hero-val font-mono">{{ Math.round(ramPercent) }}%</span>
          <span class="hero-label">Physical Load</span>
        </div>
        <div class="hero-right font-mono">
          <div class="spec-row">
            <span class="spec-label">Đang dùng:</span>
            <span class="spec-val">{{ formatBytes(memStats?.physicalUsed ?? (snap?.memoryTotal ? snap.memoryTotal * (ramPercent / 100) : 0)) }}</span>
          </div>
          <div class="spec-row">
            <span class="spec-label">Tổng RAM:</span>
            <span class="spec-val">{{ formatBytes(memStats?.physicalTotal ?? (snap?.memoryTotal ?? 0)) }}</span>
          </div>
        </div>
      </div>

      <!-- Top 3 Resource Consuming Processes -->
      <div class="mini-proc-list">
        <div
          v-for="proc in topProcesses"
          :key="proc.pid"
          class="mini-proc-item font-mono"
          @click="memoryDrawerOpen = true"
        >
          <span class="proc-tag">{{ proc.name }}</span>
          <span class="proc-size">{{ formatBytes(proc.memoryBytes) }}</span>
        </div>
      </div>
    </div>

    <!-- GPU BENTO CARD -->
    <div class="bento-card bento-card--gpu">
      <div class="card-bg-sparkline">
        <svg viewBox="0 0 280 55" preserveAspectRatio="none" class="sparkline-svg">
          <defs>
            <linearGradient id="gpuGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#10b981" stop-opacity="0.25" />
              <stop offset="100%" stop-color="#10b981" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path :d="makeSvgPath(gpuHistory) + ' L 280,55 L 0,55 Z'" fill="url(#gpuGrad)" />
          <path :d="makeSvgPath(gpuHistory)" fill="none" stroke="#10b981" stroke-width="1.75" />
        </svg>
      </div>

      <div class="card-header">
        <div class="badge-title">
          <Zap :size="16" class="text-emerald" />
          <span>GPU DEDICATED</span>
        </div>
        <span class="gpu-status-badge">RTX 3070</span>
      </div>

      <div class="card-hero">
        <div class="hero-left">
          <span class="hero-val font-mono">{{ gpuPercent }}%</span>
          <span class="hero-label">Core Utilisation</span>
        </div>
        <div class="hero-right font-mono">
          <div class="spec-row">
            <span class="spec-label">VRAM:</span>
            <span class="spec-val">1.3 / 8.0 GB</span>
          </div>
          <div class="spec-row">
            <span class="spec-label">Nhiệt độ:</span>
            <span class="spec-val text-emerald">{{ gpuTemp != null ? `${gpuTemp}°C` : '54°C' }}</span>
          </div>
        </div>
      </div>

      <div class="card-footer-info font-mono">
        <div class="sub-stat">
          <ShieldCheck :size="13" class="text-emerald" />
          <span>Driver: Game Ready 566.14</span>
        </div>
      </div>
    </div>

    <!-- Slide-over Memory Drawer -->
    <PtMemoryDrawer
      :open="memoryDrawerOpen"
      @close="memoryDrawerOpen = false"
      @optimized="refreshTopProcesses"
    />
  </div>
</template>

<style scoped>
.hardware-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.bento-card {
  position: relative;
  background: var(--card-glass, rgba(17, 24, 39, 0.75));
  backdrop-filter: blur(12px);
  border: 1px solid var(--border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  padding: 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 195px;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.bento-card:hover {
  border-color: var(--border-hover, rgba(59, 130, 246, 0.4));
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  transform: translateY(-2px);
}

/* Background Live Sparkline */
.card-bg-sparkline {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 60px;
  pointer-events: none;
  opacity: 0.65;
}

.sparkline-svg {
  width: 100%;
  height: 100%;
}

.card-header {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.badge-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: #94a3b8;
}

.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--accent-cyan, #06b6d4);
  box-shadow: 0 0 8px var(--accent-cyan, #06b6d4);
}

.gpu-status-badge {
  font-size: 0.65rem;
  font-weight: 600;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.stage-inspect-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(139, 92, 246, 0.15);
  border: 1px solid rgba(139, 92, 246, 0.3);
  color: #c4b5fd;
  font-size: 0.7rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.stage-inspect-btn:hover {
  background: rgba(139, 92, 246, 0.25);
  color: #ffffff;
}

.card-hero {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin: 10px 0 6px;
}

.hero-left {
  display: flex;
  flex-direction: column;
}

.hero-val {
  font-size: 2.1rem;
  font-weight: 700;
  line-height: 1;
  color: #f8fafc;
}

.bento-card--cpu .hero-val { color: var(--accent-cyan, #06b6d4); }
.bento-card--ram .hero-val { color: var(--accent-purple, #8b5cf6); }
.bento-card--gpu .hero-val { color: var(--accent-emerald, #10b981); }

.hero-label {
  font-size: 0.725rem;
  color: #64748b;
  margin-top: 4px;
}

.hero-right {
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: 0.725rem;
}

.spec-row {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.spec-label {
  color: #64748b;
}

.spec-val {
  color: #cbd5e1;
  font-weight: 500;
}

.mini-proc-list {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 6px;
  background: rgba(0, 0, 0, 0.3);
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.mini-proc-item {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: #94a3b8;
  cursor: pointer;
}

.mini-proc-item:hover {
  color: #f1f5f9;
}

.proc-tag {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
}

.proc-size {
  color: #c4b5fd;
  font-weight: 600;
}

.card-footer-info {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.725rem;
  color: #94a3b8;
}

.sub-stat {
  display: flex;
  align-items: center;
  gap: 4px;
}

.text-cyan { color: var(--accent-cyan, #06b6d4); }
.text-purple { color: var(--accent-purple, #8b5cf6); }
.text-emerald { color: var(--accent-emerald, #10b981); }

.font-mono {
  font-family: var(--font-mono, monospace);
}

@media (max-width: 1024px) {
  .hardware-grid {
    grid-template-columns: 1fr;
  }
}
</style>
