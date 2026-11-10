<script setup lang="ts">
import { ref } from 'vue';
import {
  AppWindow,
  CircuitBoard,
  LayoutDashboard,
  HardDrive,
  Terminal,
  Info,
  NotebookPen,
  Settings,
  Wrench,
  FolderOpen,
  Network,
  Scissors,
  FileText,
  Volume2,
  Shield,
  Play,
  Copy,
  Check,
  X,
} from '@lucide/vue';
import { QUICK_ACTIONS, type QuickActionId } from '@/lib/models/actions';
import { invoke } from '@/lib/services/api-services';
import { useExecutionLogStore } from '@/stores/execution-log-store';

const logStore = useExecutionLogStore();

const activeAction = ref<{ id: QuickActionId; labelKey: string } | null>(null);
const copied = ref(false);

const ACTION_ICONS: Record<QuickActionId, any> = {
  taskManager: AppWindow,
  deviceManager: CircuitBoard,
  controlPanel: LayoutDashboard,
  diskManagement: HardDrive,
  commandPromptAdmin: Terminal,
  powerShellAdmin: Terminal,
  systemInfo: Info,
  registryEditor: NotebookPen,
  settings: Settings,
  services: Wrench,
  fileExplorer: FolderOpen,
  networkConnections: Network,
  snippingTool: Scissors,
  notepad: FileText,
  volumeMixer: Volume2,
};

const ACTION_COMMANDS: Record<QuickActionId, string> = {
  taskManager: 'taskmgr.exe',
  deviceManager: 'devmgmt.msc',
  controlPanel: 'control.exe',
  diskManagement: 'diskmgmt.msc',
  commandPromptAdmin: 'cmd.exe',
  powerShellAdmin: 'powershell.exe',
  systemInfo: 'msinfo32.exe',
  registryEditor: 'regedit.exe',
  settings: 'start ms-settings:',
  services: 'services.msc',
  fileExplorer: 'explorer.exe',
  networkConnections: 'ncpa.cpl',
  snippingTool: 'snippingtool.exe',
  notepad: 'notepad.exe',
  volumeMixer: 'sndvol.exe',
};

const ACTION_DESCRIPTIONS: Record<QuickActionId, string> = {
  taskManager: 'Quản lý tiến trình, hiệu năng CPU/RAM và dịch vụ khởi động',
  deviceManager: 'Quản lý driver và các thiết bị phần cứng kết nối',
  controlPanel: 'Bảng điều khiển cổ điển cấu hình hệ thống Windows',
  diskManagement: 'Phân vùng, định dạng và quản lý ổ cứng vật lý',
  commandPromptAdmin: 'Cửa sổ dòng lệnh CMD với đặc quyền Administrator',
  powerShellAdmin: 'PowerShell Terminal nâng cao với quyền Quản trị viên',
  systemInfo: 'Xem chi tiết toàn bộ phần cứng và cấu hình hệ điều hành',
  registryEditor: 'Chỉnh sửa cơ sở dữ liệu đăng ký cấu hình Windows Registry',
  settings: 'Mở ứng dụng Cài đặt hiện đại (Windows Settings)',
  services: 'Bảng điều khiển các dịch vụ chạy nền của hệ thống',
  fileExplorer: 'Trình quản lý tệp và thư mục Windows Explorer',
  networkConnections: 'Bảng điều khiển card mạng và adapter kết nối',
  snippingTool: 'Công cụ chụp ảnh và quay video màn hình',
  notepad: 'Trình soạn thảo văn bản đơn giản',
  volumeMixer: 'Bộ trộn âm lượng từng ứng dụng âm thanh',
};

function openActionMenu(action: { id: QuickActionId; labelKey: string }) {
  activeAction.value = action;
  copied.value = false;
}

function closeMenu() {
  activeAction.value = null;
}

async function runAction(asAdmin = false) {
  if (!activeAction.value) return;
  const id = activeAction.value.id;
  const cmd = ACTION_COMMANDS[id] || id;

  const logId = logStore.addLog(
    `Chạy ${id} (${asAdmin ? 'Administrator' : 'Standard'})`,
    cmd,
    'running'
  );

  try {
    await invoke('open_quick_action', { id });
    logStore.updateLog(logId, {
      status: 'success',
      output: `Khởi chạy thành công: ${cmd}`,
    });
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Lỗi khởi chạy: ${err?.message || err}`,
    });
  }

  closeMenu();
}

async function copyCli() {
  if (!activeAction.value) return;
  const cmd = ACTION_COMMANDS[activeAction.value.id] || activeAction.value.id;
  await navigator.clipboard.writeText(cmd);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 2000);
}
</script>

<template>
  <div class="dock-container">
    <div class="dock-header">
      <span class="dock-title font-mono">COMPACT SYSTEM DOCK</span>
      <span class="dock-hint">Nhấn vào biểu tượng để chọn chế độ khởi chạy (Inspect & Staged Launch)</span>
    </div>

    <!-- Horizontal Icons Dock -->
    <div class="dock-icons-strip">
      <button
        v-for="action in QUICK_ACTIONS"
        :key="action.id"
        type="button"
        class="dock-icon-btn"
        :class="{ 'dock-icon-btn--active': activeAction?.id === action.id }"
        :title="action.id"
        @click="openActionMenu(action)"
      >
        <component :is="ACTION_ICONS[action.id] || AppWindow" :size="18" />
        <span class="dock-label">{{ action.id }}</span>
      </button>
    </div>

    <!-- Staged Action Popover / Inspection Modal -->
    <div v-if="activeAction" class="popover-overlay" @click.self="closeMenu">
      <div class="action-popover font-mono">
        <div class="popover-top">
          <div class="popover-title-row">
            <component :is="ACTION_ICONS[activeAction.id]" :size="18" class="text-cyan" />
            <span class="popover-name">{{ activeAction.id }}</span>
          </div>
          <button type="button" class="close-x" @click="closeMenu">
            <X :size="14" />
          </button>
        </div>

        <p class="popover-desc">{{ ACTION_DESCRIPTIONS[activeAction.id] }}</p>

        <div class="cli-command-box">
          <span class="cli-label">Lệnh thực thi:</span>
          <code>{{ ACTION_COMMANDS[activeAction.id] }}</code>
        </div>

        <div class="popover-actions">
          <button type="button" class="popover-btn" @click="copyCli">
            <Check v-if="copied" :size="14" class="text-emerald" />
            <Copy v-else :size="14" />
            <span>{{ copied ? 'Đã chép' : 'Sao chép CLI' }}</span>
          </button>
          <button type="button" class="popover-btn" @click="runAction(false)">
            <Play :size="14" />
            <span>Mở thông thường</span>
          </button>
          <button type="button" class="popover-btn popover-btn-admin" @click="runAction(true)">
            <Shield :size="14" />
            <span>Chạy quyền Admin</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dock-container {
  background: var(--card-glass, rgba(17, 24, 39, 0.75));
  backdrop-filter: blur(12px);
  border: 1px solid var(--border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  position: relative;
}

.dock-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.dock-title {
  font-size: 0.75rem;
  font-weight: 700;
  color: #94a3b8;
  letter-spacing: 0.05em;
}

.dock-hint {
  font-size: 0.7rem;
  color: #64748b;
}

.dock-icons-strip {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.dock-icon-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-width: 54px;
  height: 52px;
  padding: 6px 4px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  color: #cbd5e1;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.dock-icon-btn:hover,
.dock-icon-btn--active {
  background: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.4);
  color: #f8fafc;
  transform: translateY(-2px);
}

.dock-label {
  font-size: 0.625rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 50px;
}

/* Popover Modal */
.popover-overlay {
  position: fixed;
  inset: 0;
  z-index: 90;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-popover {
  width: 100%;
  max-width: 440px;
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  padding: 18px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.8);
  animation: popover-in 0.16s ease;
}

@keyframes popover-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.popover-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.popover-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.popover-name {
  font-size: 0.9375rem;
  font-weight: 700;
  color: #f8fafc;
}

.close-x {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px;
}

.close-x:hover {
  color: #f1f5f9;
}

.popover-desc {
  font-size: 0.775rem;
  color: #94a3b8;
  margin: 0 0 12px;
  line-height: 1.4;
  font-family: inherit;
}

.cli-command-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  margin-bottom: 14px;
}

.cli-label {
  color: #64748b;
}

.cli-command-box code {
  color: var(--accent-cyan, #06b6d4);
}

.popover-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.popover-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.04);
  color: #cbd5e1;
  cursor: pointer;
  transition: all 0.12s ease;
}

.popover-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #f8fafc;
}

.popover-btn-admin {
  background: rgba(245, 158, 11, 0.15);
  border-color: rgba(245, 158, 11, 0.35);
  color: #fcd34d;
}

.popover-btn-admin:hover {
  background: rgba(245, 158, 11, 0.25);
  color: #ffffff;
}

.text-cyan { color: var(--accent-cyan, #06b6d4); }
.text-emerald { color: #10b981; }
.font-mono { font-family: var(--font-mono, monospace); }
</style>
