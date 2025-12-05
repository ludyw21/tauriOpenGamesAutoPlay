<script setup lang="ts">
import { ref, computed, inject, watch } from "vue";
import Dialog from "../common/Dialog.vue";
import { info } from '@tauri-apps/plugin-log';
import { getNoteName, groupForNote } from "../../config/groups";

interface EventData {
  time: number;
  type: string;
  note?: number;
  channel: number;
  group?: string;
  duration?: number;
  end?: number;
}

interface EventTableDialogProps {
  visible: boolean;
  events: EventData[];
}

const props = defineProps<EventTableDialogProps>();
const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
}>();

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 获取设置
const settings = settingsManager.getSettings();
const currentMinNote = ref(settings.analyzerSetting?.minNote || 48);
const currentMaxNote = ref(settings.analyzerSetting?.maxNote || 83);

// 状态
const showOnlyOutOfRange = ref(false);
const events = computed(() => props.events || []);

// 计算超限音符数量（只统计note_on事件）
const outOfRangeCount = computed(() => {
  return events.value.filter(e => e.type === 'note_on' && e.note && isOutOfRange(e)).length;
});

// 过滤后的事件
const filteredEvents = computed(() => {
  if (showOnlyOutOfRange.value) {
    // 只显示超限音符相关的事件（包括note_on和note_off）
    return events.value.filter(e => e.note && isOutOfRange(e));
  }
  return events.value;
});

// 检查音符是否超出范围
const isOutOfRange = (event: EventData): boolean => {
  if (!event.note) return false;
  return event.note < currentMinNote.value || event.note > currentMaxNote.value;
};



// ... (imports)

// ... (props)

// ... (refs)

// ... (computed)

// ... (isOutOfRange)

// 移除本地 getNoteName 和 getNoteGroup 实现

// 切换显示模式
const toggleDisplay = () => {
  console.log('toggleDisplay called, showOnlyOutOfRange:', showOnlyOutOfRange.value);
  // v-model已经处理了值的更新，这里不需要再次切换
};

// 导出事件CSV
const exportEventCsv = () => {
  // 实现导出CSV逻辑
  info("[EventTableDialog.vue] 导出事件CSV");
};

// 导出按键谱
// const exportKeyNotation = () => {
//   // 实现导出按键谱逻辑
//   info("[EventTableDialog.vue] 导出按键谱");
// };

// 双击事件处理
const handleEventDoubleClick = (event: EventData) => {
  // 实现双击事件处理逻辑
  info(`[EventTableDialog.vue] 双击事件: ${JSON.stringify(event)}`);
};

// 监听可见性变化，初始化数据
watch(() => props.visible, (newVal) => {
  if (newVal) {
    info(`[EventTableDialog.vue] 打开事件表，事件数量: ${props.events?.length || 0}`);
  }
});
</script>

<template>
  <Dialog :visible="visible" @update:visible="emit('update:visible', $event)" title="事件表" width="800px" height="600px">
    <!-- 工具栏 -->
    <div class="event-toolbar">
      <button class="btn btn-small" @click="exportEventCsv">导出事件CSV</button>
      <!-- <button class="btn btn-small" @click="exportKeyNotation">导出按键谱</button> -->

      <div class="toolbar-right">
        <span class="out-of-range-count">超限音符数量：{{ outOfRangeCount }}</span>
        <div class="checkbox-item">
          <input type="checkbox" id="showOutOfRange" v-model="showOnlyOutOfRange" @change="toggleDisplay" />
          <label for="showOutOfRange">仅显示超限音符</label>
        </div>
      </div>
    </div>

    <!-- 事件表格 -->
    <div class="event-table-container">
      <table class="event-table">
        <thead>
          <tr>
            <th>序号</th>
            <th>时间</th>
            <th>事件</th>
            <th>音符</th>
            <th>通道</th>
            <th>分组</th>
            <th>结束</th>
            <th>时长</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(event, index) in filteredEvents" :key="index"
            :class="{ 'out-of-range': event.note && isOutOfRange(event) }" @dblclick="handleEventDoubleClick(event)">
            <td>{{ index + 1 }}</td>
            <td>{{ event.time.toFixed(2) }}</td>
            <td>{{ event.type }}</td>
            <td>{{ event.note ? getNoteName(event.note) + '(' + event.note + ')' : '-' }}</td>
            <td>{{ event.channel }}</td>
            <td>{{ event.note ? groupForNote(event.note) : '-' }}</td>
            <td>{{ event.end ? event.end.toFixed(2) : '-' }}</td>
            <td>{{ event.duration ? event.duration.toFixed(2) : '-' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </Dialog>
</template>

<style scoped>
.event-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding: 0.5rem;
  background-color: var(--active);
  border-radius: 4px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.out-of-range-count {
  font-size: 0.9rem;
  color: var(--danger);
  font-weight: 500;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.checkbox-item input[type="checkbox"] {
  accent-color: var(--primary);
}

.checkbox-item label {
  font-size: 0.9rem;
  color: var(--fg);
  cursor: pointer;
}

.event-table-container {
  max-height: 450px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 4px;
}

.event-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.event-table th {
  background-color: var(--active);
  padding: 0.5rem;
  text-align: center;
  border: 1px solid var(--border);
  font-weight: 600;
  color: var(--fg);
  position: sticky;
  top: 0;
  z-index: 10;
}

.event-table td {
  padding: 0.5rem;
  text-align: center;
  border: 1px solid var(--border);
  color: var(--fg);
}

.event-table tr:hover {
  background-color: var(--active);
}

.event-table tr.out-of-range {
  background-color: rgba(220, 53, 69, 0.1);
  color: var(--danger);
}

.event-table tr.out-of-range:hover {
  background-color: rgba(220, 53, 69, 0.2);
}

.btn-small {
  padding: 0.35rem 0.75rem;
  font-size: 0.85rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-small:hover {
  background-color: var(--primary);
  border-color: var(--primary);
  color: var(--selectfg);
}
</style>
