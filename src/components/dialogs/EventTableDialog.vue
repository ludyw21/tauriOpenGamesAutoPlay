<script setup lang="ts">
import { ref, computed, watch } from "vue";
import Dialog from "../common/Dialog.vue";

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
}

const props = defineProps<EventTableDialogProps>();
const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
}>();

// 状态
const showOnlyOutOfRange = ref(false);
const events = ref<EventData[]>([]);
const currentMinNote = ref(48);
const currentMaxNote = ref(83);

// 计算超限音符数量
const outOfRangeCount = computed(() => {
  return events.value.filter(e => e.type === 'note_on' && isOutOfRange(e)).length;
});

// 过滤后的事件
const filteredEvents = computed(() => {
  if (showOnlyOutOfRange.value) {
    return events.value.filter(e => isOutOfRange(e));
  }
  return events.value;
});

// 检查音符是否超出范围
const isOutOfRange = (event: EventData): boolean => {
  if (!event.note) return false;
  return event.note < currentMinNote.value || event.note > currentMaxNote.value;
};

// 获取音符名称
const getNoteName = (note: number): string => {
  const noteNames = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const octave = Math.floor(note / 12) - 1;
  const noteIndex = note % 12;
  return `${noteNames[noteIndex]}${octave}`;
};

// 获取音符分组（简化版本）
const getNoteGroup = (note: number): string => {
  // 这里可以根据实际的分组配置返回对应的分组名称
  // 暂时返回简单的分组逻辑
  if (note >= 48 && note <= 59) return "第一组";
  if (note >= 60 && note <= 71) return "第二组";
  if (note >= 72 && note <= 83) return "第三组";
  return "超出范围";
};

// 切换显示模式
const toggleDisplay = () => {
  showOnlyOutOfRange.value = !showOnlyOutOfRange.value;
};

// 导出事件CSV
const exportEventCsv = () => {
  // 实现导出CSV逻辑
  console.log("导出事件CSV");
};

// 导出按键谱
const exportKeyNotation = () => {
  // 实现导出按键谱逻辑
  console.log("导出按键谱");
};

// 双击事件处理
const handleEventDoubleClick = (event: EventData) => {
  // 实现双击事件处理逻辑
  console.log("双击事件:", event);
};

// 生成示例数据
const generateSampleEvents = () => {
  const sampleEvents: EventData[] = [];
  const notes = [60, 64, 67, 62, 65, 69, 45, 85]; // 包含一些超出范围的音符
  
  for (let i = 0; i < notes.length; i++) {
    const note = notes[i];
    // 添加note_on事件
    sampleEvents.push({
      time: i * 0.5,
      type: 'note_on',
      note,
      channel: 0,
      group: getNoteGroup(note),
      duration: 0.5,
      end: i * 0.5 + 0.5
    });
    // 添加note_off事件
    sampleEvents.push({
      time: i * 0.5 + 0.5,
      type: 'note_off',
      note,
      channel: 0,
      group: getNoteGroup(note)
    });
  }
  
  return sampleEvents;
};

// 监听可见性变化，初始化数据
watch(() => props.visible, (newVal) => {
  if (newVal) {
    // 加载事件数据，这里使用示例数据
    events.value = generateSampleEvents();
  }
});
</script>

<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    title="事件表"
    width="800px"
    height="600px"
  >
    <!-- 工具栏 -->
    <div class="event-toolbar">
      <button class="btn btn-small" @click="exportEventCsv">导出事件CSV</button>
      <button class="btn btn-small" @click="exportKeyNotation">导出按键谱</button>
      
      <div class="toolbar-right">
        <span class="out-of-range-count">超限音符数量：{{ outOfRangeCount }}</span>
        <div class="checkbox-item">
          <input 
            type="checkbox" 
            id="showOutOfRange" 
            v-model="showOnlyOutOfRange"
            @change="toggleDisplay"
          />
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
          <tr 
            v-for="(event, index) in filteredEvents" 
            :key="index"
            :class="{ 'out-of-range': event.note && isOutOfRange(event) }"
            @dblclick="handleEventDoubleClick(event)"
          >
            <td>{{ index + 1 }}</td>
            <td>{{ event.time.toFixed(2) }}</td>
            <td>{{ event.type }}</td>
            <td>{{ event.note ? getNoteName(event.note) + '(' + event.note + ')' : '-' }}</td>
            <td>{{ event.channel }}</td>
            <td>{{ event.note ? getNoteGroup(event.note) : '-' }}</td>
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