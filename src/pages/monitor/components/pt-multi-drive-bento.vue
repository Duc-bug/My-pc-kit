<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import {
  HardDrive,
  Search,
  Trash2,
  FolderTree,
  FileCode,
  FileArchive,
  CheckCircle2,
  AlertTriangle,
  RotateCw,
  Folder,
  File,
  ShieldAlert,
} from '@lucide/vue';
import { formatBytes } from '@/lib/utils/format';
import {
  DriveCleanerService,
  type DriveCleanItem,
  type DiskInfo,
} from '@/lib/services/api-services';
import { useMonitorStore } from '@/stores/monitor-store';
import { useExecutionLogStore } from '@/stores/execution-log-store';
import PtConfirmDialog from '@/components/custom/pt-confirm-dialog.vue';

const monitor = useMonitorStore();
const logStore = useExecutionLogStore();

const currentDriveIndex = ref(0);
const scanning = ref(false);
const cleaning = ref(false);
const scanResultItems = ref<DriveCleanItem[]>([]);
const selectedItemIds = ref<string[]>([]);
const confirmDialogOpen = ref(false);

const activeFilterCategory = ref<'all' | 'system_temp' | 'dev_gamer' | 'large_file'>('all');

function getDiskLetter(disk?: DiskInfo): string {
  if (!disk) return 'C:';
  const name = disk.mountPoint || disk.name || 'C:';
  return name.replace(/\\+$/, '');
}

const disks = computed<DiskInfo[]>(() => {
  const list = monitor.snapshot?.disks;
  if (list && list.length > 0) return list;
  // Fallback
  return [
    {
      name: 'C:',
      mountPoint: 'C:\\',
      totalSpace: 512000000000,
      usedSpace: 390000000000,
      availableSpace: 122000000000,
      percent: 76.1,
    },
  ];
});

const currentDisk = computed(() => {
  return disks.value[currentDriveIndex.value] ?? disks.value[0];
});

const filteredItems = computed(() => {
  if (activeFilterCategory.value === 'all') return scanResultItems.value;
  return scanResultItems.value.filter((item) => item.category === activeFilterCategory.value);
});

const selectedBytes = computed(() => {
  return scanResultItems.value
    .filter((item) => selectedItemIds.value.includes(item.id))
    .reduce((acc, item) => acc + item.sizeBytes, 0);
});

// Auto scan on drive change or initial mount
async function handleScanDrive() {
  if (scanning.value) return;
  scanning.value = true;
  scanResultItems.value = [];
  selectedItemIds.value = [];

  const drive = getDiskLetter(currentDisk.value);
  const logId = logStore.addLog(
    `Quét phân tích ổ đĩa ${drive}`,
    `DriveCleanerService.scanDrive("${drive}")`,
    'running'
  );

  try {
    const res = await DriveCleanerService.scanDrive(drive);
    scanResultItems.value = res.items;
    // Auto select temp and dev caches by default (exclude large files by default for safety)
    selectedItemIds.value = res.items
      .filter((i) => i.category !== 'large_file')
      .map((i) => i.id);

    logStore.updateLog(logId, {
      status: 'success',
      output: `Quét hoàn tất: Tìm thấy ${res.items.length} mục (${formatBytes(res.totalBytes)}) trên ổ ${drive}`,
    });
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Lỗi quét ổ đĩa: ${err?.message || err}`,
    });
  } finally {
    scanning.value = false;
  }
}

watch(currentDriveIndex, () => {
  void handleScanDrive();
});

onMounted(() => {
  void handleScanDrive();
});

function toggleItem(id: string) {
  const idx = selectedItemIds.value.indexOf(id);
  if (idx !== -1) {
    selectedItemIds.value.splice(idx, 1);
  } else {
    selectedItemIds.value.push(id);
  }
}

function selectAllFiltered() {
  const ids = filteredItems.value.map((i) => i.id);
  selectedItemIds.value = Array.from(new Set([...selectedItemIds.value, ...ids]));
}

function deselectAllFiltered() {
  const idsToRemove = new Set(filteredItems.value.map((i) => i.id));
  selectedItemIds.value = selectedItemIds.value.filter((id) => !idsToRemove.has(id));
}

function promptConfirmClean() {
  if (selectedItemIds.value.length === 0) return;
  confirmDialogOpen.value = true;
}

/** STEP 2: Execute Clean & stream execution log */
async function executeCleanConfirmed() {
  confirmDialogOpen.value = false;
  if (cleaning.value || selectedItemIds.value.length === 0) return;
  cleaning.value = true;

  const targets = scanResultItems.value.filter((i) => selectedItemIds.value.includes(i.id));
  const paths = targets.map((t) => t.path);

  const logId = logStore.addLog(
    `Dọn dẹp ${paths.length} mục trên ổ ${getDiskLetter(currentDisk.value)}`,
    `rmdir /s /q [${paths.length} targets]`,
    'running'
  );

  try {
    const res = await DriveCleanerService.executeCleanup(paths);
    logStore.updateLog(logId, {
      status: res.success ? 'success' : 'error',
      output: res.executionLog.join('\n'),
    });

    // Remove deleted items from UI
    scanResultItems.value = scanResultItems.value.filter(
      (item) => !selectedItemIds.value.includes(item.id)
    );
    selectedItemIds.value = [];
    await monitor.refresh();
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Lỗi thực thi dọn dẹp: ${err?.message || err}`,
    });
  } finally {
    cleaning.value = false;
  }
}
</script>

<template>
  <div class="multi-drive-card">
    <div class="drive-header">
      <div class="header-title">
        <HardDrive :size="16" class="text-amber" />
        <span>MULTI-DRIVE MANAGER & STAGED CLEANER</span>
      </div>

      <!-- Segmented Drive Switcher Tabs -->
      <div class="segmented-drive-tabs font-mono">
        <button
          v-for="(disk, idx) in disks"
          :key="getDiskLetter(disk)"
          type="button"
          class="drive-tab-btn"
          :class="{ 'drive-tab-btn--active': currentDriveIndex === idx }"
          @click="currentDriveIndex = idx"
        >
          <span class="tab-dev">{{ getDiskLetter(disk) }}</span>
          <span class="tab-pct">{{ disk.percent.toFixed(0) }}%</span>
        </button>
      </div>
    </div>

    <!-- Active Drive Capacity & Speed Bar -->
    <div class="drive-capacity-strip font-mono">
      <div class="capacity-info">
        <span class="drive-badge">{{ getDiskLetter(currentDisk) }} (NTFS)</span>
        <span class="capacity-text">
          {{ formatBytes(currentDisk.usedSpace) }} / {{ formatBytes(currentDisk.totalSpace) }}
          <span class="free-text">(Trống: {{ formatBytes(currentDisk.availableSpace) }})</span>
        </span>
      </div>
      <div class="capacity-meter">
        <div
          class="capacity-meter-fill"
          :style="{ width: `${Math.min(100, Math.max(0, currentDisk.percent))}%` }"
        />
      </div>
    </div>

    <!-- 2-Step Staged Filter & Action Bar -->
    <div class="scan-controls-bar">
      <div class="filter-pills font-mono">
        <button
          type="button"
          class="filter-pill"
          :class="{ 'filter-pill--active': activeFilterCategory === 'all' }"
          @click="activeFilterCategory = 'all'"
        >
          Tất cả ({{ scanResultItems.length }})
        </button>
        <button
          type="button"
          class="filter-pill"
          :class="{ 'filter-pill--active': activeFilterCategory === 'system_temp' }"
          @click="activeFilterCategory = 'system_temp'"
        >
          <FolderTree :size="12" /> Temp / Logs
        </button>
        <button
          type="button"
          class="filter-pill"
          :class="{ 'filter-pill--active': activeFilterCategory === 'dev_gamer' }"
          @click="activeFilterCategory = 'dev_gamer'"
        >
          <FileCode :size="12" /> Dev & Shader
        </button>
        <button
          type="button"
          class="filter-pill"
          :class="{ 'filter-pill--active': activeFilterCategory === 'large_file' }"
          @click="activeFilterCategory = 'large_file'"
        >
          <FileArchive :size="12" /> File lớn (>1GB)
        </button>
      </div>

      <div class="stage-actions">
        <button
          type="button"
          class="pt-btn pt-btn-secondary"
          :disabled="scanning"
          @click="handleScanDrive"
        >
          <RotateCw :size="14" :class="{ spin: scanning }" />
          {{ scanning ? 'Đang phân tích…' : 'Quét phân tích' }}
        </button>
        <button
          type="button"
          class="pt-btn pt-btn-amber"
          :disabled="selectedItemIds.length === 0 || cleaning"
          @click="promptConfirmClean"
        >
          <Trash2 :size="14" />
          Xác nhận dọn dẹp ({{ formatBytes(selectedBytes) }})
        </button>
      </div>
    </div>

    <!-- Scan Results Tree / List -->
    <div class="scan-results-box">
      <div v-if="scanning" class="state-msg font-mono">
        <RotateCw :size="20" class="spin text-cyan" />
        <span>Đang quét sâu phân vùng {{ getDiskLetter(currentDisk) }}...</span>
      </div>

      <div v-else-if="filteredItems.length === 0" class="state-msg">
        <CheckCircle2 :size="20" class="text-emerald" />
        <span>Không tìm thấy mục rác hoặc file thừa trong danh mục này.</span>
      </div>

      <div v-else class="items-list font-mono">
        <div class="list-selection-bar">
          <button type="button" class="text-btn" @click="selectAllFiltered">Chọn danh mục này</button>
          <span class="dot">•</span>
          <button type="button" class="text-btn" @click="deselectAllFiltered">Bỏ chọn</button>
          <span class="selection-summary">Đã chọn {{ selectedItemIds.length }} / {{ scanResultItems.length }} mục</span>
        </div>

        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="clean-item-row"
          :class="{ 'clean-item-row--checked': selectedItemIds.includes(item.id) }"
          @click="toggleItem(item.id)"
        >
          <input
            type="checkbox"
            :checked="selectedItemIds.includes(item.id)"
            class="item-checkbox"
            @click.stop="toggleItem(item.id)"
          />

          <div class="item-icon-box">
            <Folder v-if="item.isDir" :size="15" class="text-amber" />
            <File v-else :size="15" class="text-cyan" />
          </div>

          <div class="item-details">
            <span class="item-name">{{ item.name }}</span>
            <span class="item-path">{{ item.path }}</span>
          </div>

          <div class="item-meta">
            <span class="item-size">{{ formatBytes(item.sizeBytes) }}</span>
            <span v-if="item.itemCount > 1" class="item-files-count">({{ item.itemCount }} files)</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Step 2 Final Confirm Modal -->
    <PtConfirmDialog
      v-model:open="confirmDialogOpen"
      :title="`Xác nhận dọn dẹp ổ đĩa ${getDiskLetter(currentDisk)}`"
      :message="`Bạn có chắc chắn muốn xóa ${selectedItemIds.length} mục đã chọn? Dự kiến giải phóng khoảng ${formatBytes(selectedBytes)}. Mọi thao tác xóa sẽ được in vào System Action Log.`"
      confirm-text="Thực thi xóa ngay"
      cancel-text="Hủy bỏ"
      @confirm="executeCleanConfirmed"
    />
  </div>
</template>

<style scoped>
.multi-drive-card {
  background: var(--card-glass, rgba(17, 24, 39, 0.75));
  backdrop-filter: blur(12px);
  border: 1px solid var(--border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  transition: all 0.2s ease;
}

.multi-drive-card:hover {
  border-color: var(--border-hover, rgba(59, 130, 246, 0.4));
}

.drive-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  color: #cbd5e1;
}

.segmented-drive-tabs {
  display: flex;
  gap: 4px;
  background: rgba(15, 23, 42, 0.8);
  padding: 3px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.drive-tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #94a3b8;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.drive-tab-btn:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.05);
}

.drive-tab-btn--active {
  background: var(--accent-amber, #f59e0b) !important;
  color: #090d16 !important;
  font-weight: 700;
}

.drive-capacity-strip {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.capacity-info {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  font-size: 0.75rem;
}

.drive-badge {
  color: #f1f5f9;
  font-weight: 600;
}

.capacity-text {
  color: #cbd5e1;
}

.free-text {
  color: #94a3b8;
  margin-left: 4px;
}

.capacity-meter {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.capacity-meter-fill {
  height: 100%;
  background: linear-gradient(90deg, #f59e0b, #ef4444);
  border-radius: 999px;
  transition: width 0.3s ease;
}

.scan-controls-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-pills {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.filter-pill {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 6px;
  color: #94a3b8;
  font-size: 0.725rem;
  cursor: pointer;
  transition: all 0.12s ease;
}

.filter-pill:hover,
.filter-pill--active {
  background: rgba(56, 189, 248, 0.12);
  border-color: rgba(56, 189, 248, 0.3);
  color: #f1f5f9;
}

.filter-pill--active {
  font-weight: 600;
}

.stage-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pt-btn-amber {
  background: #f59e0b;
  color: #090d16;
  font-weight: 600;
}

.pt-btn-amber:hover:not(:disabled) {
  background: #d97706;
}

.scan-results-box {
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  min-height: 160px;
  max-height: 240px;
  overflow-y: auto;
  padding: 10px;
}

.state-msg {
  height: 140px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #94a3b8;
  font-size: 0.8125rem;
}

.list-selection-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.725rem;
  color: #64748b;
}

.text-btn {
  background: transparent;
  border: none;
  color: #38bdf8;
  cursor: pointer;
  padding: 0;
}

.text-btn:hover {
  text-decoration: underline;
}

.selection-summary {
  margin-left: auto;
  color: #94a3b8;
}

.clean-item-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.1s ease;
}

.clean-item-row:hover,
.clean-item-row--checked {
  background: rgba(255, 255, 255, 0.04);
}

.clean-item-row--checked {
  border-color: rgba(245, 158, 11, 0.25);
}

.item-checkbox {
  cursor: pointer;
}

.item-icon-box {
  flex-shrink: 0;
}

.item-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.item-name {
  font-size: 0.75rem;
  font-weight: 500;
  color: #f1f5f9;
}

.item-path {
  font-size: 0.6875rem;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-meta {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 0.75rem;
  color: var(--accent-amber, #f59e0b);
  font-weight: 600;
}

.item-files-count {
  font-size: 0.6875rem;
  color: #64748b;
}

.text-amber { color: var(--accent-amber, #f59e0b); }
.text-cyan { color: var(--accent-cyan, #06b6d4); }
.text-emerald { color: var(--accent-emerald, #10b981); }

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
