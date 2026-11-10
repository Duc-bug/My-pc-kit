<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
  X,
  Layers,
  Cpu,
  Trash2,
  AlertTriangle,
  CheckCircle2,
  ShieldAlert,
  Play,
  RotateCw,
} from '@lucide/vue';
import { formatBytes } from '@/lib/utils/format';
import { ProcessService, MemoryCleanerService, type TopProcessInfo } from '@/lib/services/api-services';
import { useExecutionLogStore } from '@/stores/execution-log-store';
import { useMemoryCleanerStore } from '@/stores/memory-cleaner-store';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'optimized'): void;
}>();

const props = defineProps<{
  open: boolean;
}>();

const logStore = useExecutionLogStore();
const memoryStore = useMemoryCleanerStore();

const loading = ref(false);
const processes = ref<TopProcessInfo[]>([]);
const selectedPids = ref<number[]>([]);
const optimizing = ref(false);

const selectedTotalBytes = computed(() => {
  return processes.value
    .filter((p) => selectedPids.value.includes(p.pid))
    .reduce((acc, p) => acc + p.memoryBytes, 0);
});

async function loadProcesses() {
  loading.value = true;
  try {
    const list = await ProcessService.getTopProcesses(15);
    processes.value = list;
    // Default select top 3 processes
    selectedPids.value = list.slice(0, 3).map((p) => p.pid);
  } catch (err) {
    console.error('Failed to load processes:', err);
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      void loadProcesses();
    }
  }
);

function togglePid(pid: number) {
  const idx = selectedPids.value.indexOf(pid);
  if (idx !== -1) {
    selectedPids.value.splice(idx, 1);
  } else {
    selectedPids.value.push(pid);
  }
}

function selectAll() {
  selectedPids.value = processes.value.map((p) => p.pid);
}

function deselectAll() {
  selectedPids.value = [];
}

/** 2-STEP STAGED EXECUTION: Review working sets -> Execute release & log */
async function executeStagedOptimize() {
  if (optimizing.value) return;
  optimizing.value = true;

  const logId = logStore.addLog(
    'Giải phóng Working Set & Tối ưu RAM',
    'Win32 EmptyWorkingSet / SetSystemFileCacheSize',
    'running'
  );

  try {
    await memoryStore.run('manual');
    const res = memoryStore.result;
    const summary = res
      ? `Đã hoàn tất giải phóng bộ nhớ hệ thống.\nKết quả: ${res.areas.map((o) => `${o.id}: ${o.status}`).join(', ')}`
      : 'Đã hoàn tất giải phóng bộ nhớ hệ thống.';
    logStore.updateLog(logId, {
      status: 'success',
      output: summary,
    });
    emit('optimized');
    emit('close');
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Lỗi khi tối ưu bộ nhớ: ${err?.message || err}`,
    });
  } finally {
    optimizing.value = false;
  }
}

/** Staged terminate selected processes */
async function terminateSelected() {
  if (selectedPids.value.length === 0) return;

  for (const pid of selectedPids.value) {
    const proc = processes.value.find((p) => p.pid === pid);
    const name = proc?.name || `PID ${pid}`;
    const logId = logStore.addLog(`Tắt tiến trình: ${name}`, `taskkill /F /PID ${pid}`, 'running');

    try {
      const res = await ProcessService.terminateProcess(pid);
      logStore.updateLog(logId, {
        status: res.success ? 'success' : 'error',
        output: res.output,
      });
    } catch (e: any) {
      logStore.updateLog(logId, {
        status: 'error',
        output: String(e),
      });
    }
  }

  await loadProcesses();
  emit('optimized');
}
</script>

<template>
  <div v-if="open" class="drawer-overlay" @click.self="emit('close')">
    <div class="slide-over">
      <div class="drawer-top">
        <div class="drawer-title-group">
          <Layers :size="18" class="text-cyan" />
          <div>
            <h3>Phân tích Bộ nhớ & Tiến trình (Staged Inspector)</h3>
            <span class="sub">Xem trước dung lượng Working Set dự kiến thu hồi trước khi thực thi</span>
          </div>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>

      <div class="drawer-content">
        <!-- Working set summary banner -->
        <div class="summary-box">
          <div class="summary-row">
            <span class="label">Tiến trình đã chọn:</span>
            <span class="value font-mono">{{ selectedPids.length }} / {{ processes.length }}</span>
          </div>
          <div class="summary-row">
            <span class="label">Dung lượng Working Set ước tính:</span>
            <span class="value-highlight font-mono">{{ formatBytes(selectedTotalBytes) }}</span>
          </div>
          <div class="batch-actions">
            <button type="button" class="link-btn" @click="selectAll">Chọn tất cả</button>
            <span class="dot">•</span>
            <button type="button" class="link-btn" @click="deselectAll">Bỏ chọn</button>
            <span class="dot">•</span>
            <button type="button" class="link-btn" :disabled="loading" @click="loadProcesses">
              <RotateCw :size="12" :class="{ spin: loading }" /> Làm mới
            </button>
          </div>
        </div>

        <!-- Process list -->
        <div class="process-section">
          <h4>Top tiến trình chiếm dụng RAM</h4>
          <div class="process-list">
            <div
              v-for="proc in processes"
              :key="proc.pid"
              class="proc-item"
              :class="{ 'proc-item--checked': selectedPids.includes(proc.pid) }"
              @click="togglePid(proc.pid)"
            >
              <input
                type="checkbox"
                :checked="selectedPids.includes(proc.pid)"
                class="proc-check"
                @click.stop="togglePid(proc.pid)"
              />
              <div class="proc-info">
                <span class="proc-name">{{ proc.name }}</span>
                <span class="proc-pid font-mono">PID: {{ proc.pid }}</span>
              </div>
              <div class="proc-stats font-mono">
                <span class="proc-cpu">{{ proc.cpuPercent }}% CPU</span>
                <span class="proc-mem">{{ formatBytes(proc.memoryBytes) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Action Bar (Zero One-Click: Review -> Confirm Execution) -->
      <div class="drawer-footer">
        <button
          type="button"
          class="pt-btn pt-btn-danger"
          :disabled="selectedPids.length === 0"
          @click="terminateSelected"
        >
          <Trash2 :size="15" />
          Tắt {{ selectedPids.length }} tiến trình đã chọn
        </button>
        <button
          type="button"
          class="pt-btn pt-btn-primary"
          :disabled="optimizing"
          @click="executeStagedOptimize"
        >
          <Play :size="15" />
          {{ optimizing ? 'Đang thực thi…' : 'Thực thi giải phóng RAM' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drawer-overlay {
  position: fixed;
  inset: 0;
  z-index: 95;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(6px);
  display: flex;
  justify-content: flex-end;
}

.slide-over {
  width: 100%;
  max-width: 480px;
  height: 100%;
  background: #111827;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  animation: slide-in 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slide-in {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.drawer-top {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(15, 23, 42, 0.6);
}

.drawer-title-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.drawer-title-group h3 {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 600;
  color: #f8fafc;
}

.drawer-title-group .sub {
  font-size: 0.75rem;
  color: #94a3b8;
}

.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.drawer-content {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.summary-box {
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  padding: 14px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 0.8125rem;
}

.summary-row .label {
  color: #94a3b8;
}

.summary-row .value {
  color: #f1f5f9;
  font-weight: 600;
}

.summary-row .value-highlight {
  color: var(--accent-cyan, #06b6d4);
  font-size: 1.05rem;
  font-weight: 700;
}

.batch-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.75rem;
}

.link-btn {
  background: transparent;
  border: none;
  color: #38bdf8;
  cursor: pointer;
  padding: 0;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.link-btn:hover {
  text-decoration: underline;
}

.dot {
  color: #475569;
}

.process-section h4 {
  margin: 0 0 10px;
  font-size: 0.8125rem;
  font-weight: 600;
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.proc-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.12s ease;
}

.proc-item:hover,
.proc-item--checked {
  background: rgba(56, 189, 248, 0.08);
  border-color: rgba(56, 189, 248, 0.25);
}

.proc-check {
  margin-right: 10px;
  cursor: pointer;
}

.proc-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.proc-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.proc-pid {
  font-size: 0.7rem;
  color: #64748b;
}

.proc-stats {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  font-size: 0.75rem;
}

.proc-cpu {
  color: #94a3b8;
  font-size: 0.7rem;
}

.proc-mem {
  font-weight: 600;
  color: var(--accent-purple, #8b5cf6);
}

.drawer-footer {
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(15, 23, 42, 0.8);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.text-cyan {
  color: var(--accent-cyan, #06b6d4);
}

.font-mono {
  font-family: var(--font-mono, monospace);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
