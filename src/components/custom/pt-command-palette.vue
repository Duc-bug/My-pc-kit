<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import {
  Search,
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
  CornerDownLeft,
  Shield,
  Copy,
  ExternalLink,
} from '@lucide/vue';
import { QUICK_ACTIONS, type QuickActionId } from '@/lib/models/actions';
import { invoke } from '@/lib/services/api-services';
import { useExecutionLogStore } from '@/stores/execution-log-store';

function isAdminAction(id: QuickActionId): boolean {
  return id.toLowerCase().includes('admin') || id === 'registryEditor' || id === 'services' || id === 'diskManagement';
}

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const props = defineProps<{
  open: boolean;
}>();

const logStore = useExecutionLogStore();
const query = ref('');
const selectedIndex = ref(0);
const searchInput = ref<HTMLInputElement | null>(null);

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

const filteredActions = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return QUICK_ACTIONS;
  return QUICK_ACTIONS.filter(
    (a) =>
      a.id.toLowerCase().includes(q) ||
      ACTION_COMMANDS[a.id]?.toLowerCase().includes(q) ||
      (isAdminAction(a.id) && 'admin'.includes(q))
  );
});

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      query.value = '';
      selectedIndex.value = 0;
      nextTick(() => {
        searchInput.value?.focus();
      });
    }
  }
);

watch(filteredActions, () => {
  selectedIndex.value = 0;
});

function handleKeydown(e: KeyboardEvent) {
  if (!props.open) return;

  if (e.key === 'Escape') {
    emit('close');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (filteredActions.value.length > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % filteredActions.value.length;
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (filteredActions.value.length > 0) {
      selectedIndex.value =
        (selectedIndex.value - 1 + filteredActions.value.length) % filteredActions.value.length;
    }
  } else if (e.key === 'Enter') {
    e.preventDefault();
    const action = filteredActions.value[selectedIndex.value];
    if (action) {
      const asAdmin = e.ctrlKey || isAdminAction(action.id);
      executeAction(action.id, asAdmin);
    }
  }
}

async function executeAction(id: QuickActionId, asAdmin = false) {
  const cmd = ACTION_COMMANDS[id] || id;
  const logId = logStore.addLog(
    `Khởi chạy ${id} (${asAdmin ? 'Admin' : 'Normal'})`,
    cmd,
    'running'
  );

  try {
    await invoke('open_quick_action', { id });
    logStore.updateLog(logId, {
      status: 'success',
      output: `Launched ${id} successfully with executable: ${cmd}`,
    });
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Failed to launch ${id}: ${err?.message || err}`,
    });
  }

  emit('close');
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <div v-if="open" class="palette-overlay" @click.self="emit('close')">
    <div class="palette-modal">
      <div class="palette-search">
        <Search :size="18" class="search-icon" />
        <input
          ref="searchInput"
          v-model="query"
          type="text"
          placeholder="Tìm công cụ hệ thống (Gõ tên lệnh hoặc phím mũi tên)..."
          class="palette-input"
        />
        <kbd class="esc-kbd" @click="emit('close')">ESC</kbd>
      </div>

      <div class="palette-list">
        <div
          v-for="(action, idx) in filteredActions"
          :key="action.id"
          class="palette-item"
          :class="{ 'palette-item--selected': idx === selectedIndex }"
          @mouseenter="selectedIndex = idx"
          @click="executeAction(action.id, isAdminAction(action.id))"
        >
          <div class="item-left">
            <component :is="ACTION_ICONS[action.id] || AppWindow" :size="16" class="item-icon" />
            <div class="item-info">
              <span class="item-name">{{ action.id }}</span>
              <span class="item-cmd font-mono">{{ ACTION_COMMANDS[action.id] }}</span>
            </div>
          </div>

          <div class="item-right">
            <span v-if="isAdminAction(action.id)" class="admin-badge">
              <Shield :size="11" /> Admin
            </span>
            <div class="kbd-hint">
              <span>Enter</span>
              <CornerDownLeft :size="12" />
            </div>
          </div>
        </div>

        <div v-if="filteredActions.length === 0" class="empty-state">
          Không tìm thấy công cụ phù hợp với "{{ query }}"
        </div>
      </div>

      <div class="palette-footer">
        <div class="footer-hint">
          <kbd>↑</kbd><kbd>↓</kbd> để duyệt
          <span class="divider">•</span>
          <kbd>Enter</kbd> mở công cụ
          <span class="divider">•</span>
          <kbd>Ctrl + Enter</kbd> chạy quyền Admin
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 14vh;
}

.palette-modal {
  width: 100%;
  max-width: 580px;
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(56, 189, 248, 0.2);
  overflow: hidden;
  animation: modal-pop 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modal-pop {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(-8px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.palette-search {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(15, 23, 42, 0.8);
}

.search-icon {
  color: var(--accent-cyan, #06b6d4);
  flex-shrink: 0;
}

.palette-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #f8fafc;
  font-size: 0.9375rem;
  font-family: inherit;
}

.palette-input::placeholder {
  color: #64748b;
}

.esc-kbd {
  font-size: 0.65rem;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #94a3b8;
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  font-family: var(--font-mono);
}

.palette-list {
  max-height: 340px;
  overflow-y: auto;
  padding: 6px;
}

.palette-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.palette-item:hover,
.palette-item--selected {
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.item-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.item-icon {
  color: #94a3b8;
}

.palette-item--selected .item-icon {
  color: var(--accent-cyan, #06b6d4);
}

.item-info {
  display: flex;
  flex-direction: column;
}

.item-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: #f1f5f9;
}

.item-cmd {
  font-size: 0.7rem;
  color: #64748b;
}

.item-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.admin-badge {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 0.65rem;
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.25);
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
}

.kbd-hint {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 0.65rem;
  color: #64748b;
}

.empty-state {
  padding: 28px;
  text-align: center;
  font-size: 0.8125rem;
  color: #64748b;
}

.palette-footer {
  padding: 8px 16px;
  background: rgba(15, 23, 42, 0.9);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.725rem;
  color: #64748b;
}

.footer-hint {
  display: flex;
  align-items: center;
  gap: 6px;
}

kbd {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
  padding: 1px 5px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-family: var(--font-mono);
}

.divider {
  color: #334155;
}

.font-mono {
  font-family: var(--font-mono, monospace);
}
</style>
