```vue
<script setup lang="ts">
import { ref, defineProps, watch, onMounted, onUnmounted, computed, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import EventTableDialog from "./dialogs/EventTableDialog.vue";
import SettingsDialog from "./dialogs/SettingsDialog.vue";
import HelpDialog from "./dialogs/HelpDialog.vue";
import { info, error } from '@tauri-apps/plugin-log';
import { getNoteName, groupForNote } from "../config/groups";
import { NOTE_TO_KEY } from "../config/keyboard_mapping";

const props = defineProps({
  selectedMidiFile: { type: [String, null], default: null },
});

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 右侧面板组件
const currentMinNote = ref(48);
const currentMaxNote = ref(83);

// 初始化时从 settingsManager 获取配置
onMounted(() => {
  const settings = settingsManager.getSettings();
  currentMinNote.value = settings.keySettings?.minNote || 48;
  currentMaxNote.value = settings.keySettings?.maxNote || 83;
});

// 组件卸载时清理
onUnmounted(() => {
  // 停止MIDI播放（包括预览）
  if (isPlayingMidi.value) {
    stopMidiPlayback();
  }
});

const remainingTime = ref("00:00");
const allTracksSelected = ref(true);

interface TrackAnalysis {
  max_note: number | null;
  min_note: number | null;
  max_note_name: string;
  min_note_name: string;
  max_note_group: string;
  min_note_group: string;
  upper_over_limit: number;
  lower_over_limit: number;
  is_max_over_limit: boolean;
  is_min_over_limit: boolean;
  suggested_max_transpose: number | null;
  suggested_max_octave: number | null;
  suggested_min_transpose: number | null;
  suggested_min_octave: number | null;
}

interface Track {
  id: number;
  name: string;
  noteCount: number;
  selected: boolean;
  transpose: number;
  octave: number;
  analysis: TrackAnalysis;
}

const tracks = ref<Track[]>([]);
const displayFileName = ref("");
const midiEvents = ref<any[]>([]);
const originalMidiEvents = ref<any[]>([]); // 保存原始事件数据

// 监听文件选择变化
watch(() => props.selectedMidiFile, async (newFile) => {
  // 切换歌曲时停止所有播放、预览和倒计时
  if (isPlaying.value || countdownSeconds.value > 0) {
    await stopPlayback();
  }
  if (isPlayingMidi.value || isPreviewing.value) {
    await stopMidiPlayback();
  }

  if (newFile) {
    // 提取文件名用于显示
    displayFileName.value = newFile.split(/[/\\]/).pop() || newFile;

    try {
      info("[RightPanel.vue:22] 正在解析MIDI文件: " + newFile);

      // 从设置中获取min/max note
      const settings = settingsManager.getSettings();
      const minNote = settings.keySettings?.minNote || 48;
      const maxNote = settings.keySettings?.maxNote || 83;

      // 更新当前显示的范围（确保界面显示正确的范围）
      currentMinNote.value = minNote;
      currentMaxNote.value = maxNote;

      // 传递min/max note给后端
      const result: any = await invoke("parse_midi", {
        filePath: newFile,
        minNote: minNote,
        maxNote: maxNote,
        blackKeyMode: settings.keySettings?.blackKeyMode || "support_black_key",
        trimLongNotes: settings.keySettings?.trimLongNotes || false
      });
      info("[RightPanel.vue:33] 解析成功");

      // 更新分析结果
      // 注意：这里不再覆盖 currentMinNote/MaxNote，因为它们由用户设置决定
      // 但我们可以显示 MIDI 文件的实际范围作为参考，或者仅在首次加载时设置？
      // 根据需求，min/max note 是用户设定的显示/播放范围，而不是 MIDI 文件的实际范围
      // 所以这里不需要更新 currentMinNote/MaxNote

      // 更新事件数据
      if (result.events) {
        originalMidiEvents.value = JSON.parse(JSON.stringify(result.events)); // 深拷贝
        midiEvents.value = result.events;
      } else {
        originalMidiEvents.value = [];
        midiEvents.value = [];
      }

      // 更新音轨列表
      if (result.tracks) {
        tracks.value = result.tracks.map((t: any) => ({
          id: t.id,
          name: t.name,
          noteCount: t.note_count,
          selected: true, // 默认全选
          transpose: 0,
          octave: 0,
          analysis: t.analysis // 使用后端返回的详细分析
        }));
        allTracksSelected.value = true;
      }

    } catch (e) {
      error(`[RightPanel.vue:44] 解析MIDI失败: ${e}`);
      tracks.value = [];
      midiEvents.value = [];
      originalMidiEvents.value = [];
    }

    // 启用播放相关按钮
    enablePlaybackButtons();
  } else {
    tracks.value = [];
    midiEvents.value = [];
    originalMidiEvents.value = [];

    // 禁用所有按钮
    isPlayButtonEnabled.value = false;
    isStopButtonEnabled.value = false;
    isPreviewButtonEnabled.value = false;
    isMidiPlaybackButtonEnabled.value = false;
  }
});

// 监听音轨的transpose和octave变化（用于手动输入时触发重新分析）
watch(() => tracks.value.map(t => ({ id: t.id, transpose: t.transpose, octave: t.octave })), (newValues, oldValues) => {
  if (oldValues && oldValues.length > 0) {
    newValues.forEach((newVal, index) => {
      const oldVal = oldValues[index];
      if (oldVal && (newVal.transpose !== oldVal.transpose || newVal.octave !== oldVal.octave)) {
        const track = tracks.value.find(t => t.id === newVal.id);
        if (track) {
          // 延迟执行以避免在adjustTranspose/adjustOctave中重复调用
          setTimeout(() => reanalyzeTrack(track), 0);
        }
      }
    });
  }
}, { deep: true });



// ... (imports)

// ... (props)

// ... (refs)

// ... (watch)

// 移除本地 getNoteName 实现，直接使用导入的函数

// 根据选中的音轨过滤事件
const filteredMidiEvents = computed(() => {
  const selectedTrackIds = new Set(tracks.value.filter(t => t.selected).map(t => t.id));
  return midiEvents.value.filter(event => selectedTrackIds.has(event.track));
});

// 切换全选
const toggleSelectAll = () => {
  tracks.value.forEach(track => {
    track.selected = allTracksSelected.value;
  });
  info(`[RightPanel.vue:55] 全选状态更新: ${allTracksSelected.value ? '全选' : '取消全选'}，当前选中${tracks.value.filter(t => t.selected).length}个音轨`);
};

// 切换音轨选择
const toggleTrackSelection = (trackId: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    // 不需要手动切换状态，因为v-model已经处理了
    // 只需要更新全选状态
    allTracksSelected.value = tracks.value.every(t => t.selected);
    info(`[RightPanel.vue:66] 音轨${trackId}选择状态: ${track.selected ? '选中' : '取消选中'}`);
  }
};

// 调整移调
const adjustTranspose = (trackId: number, delta: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.transpose += delta;
    // 转音设置变更后重新分析
    reanalyzeTrack(track);
  }
};

// 调整转位
const adjustOctave = (trackId: number, delta: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.octave += delta;
    // 转音设置变更后重新分析
    reanalyzeTrack(track);
  }
};

// 重置转音
const resetTranspose = (trackId: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.transpose = 0;
    track.octave = 0;
    // 转音设置变更后重新分析
    reanalyzeTrack(track);
  }
};

// 重新分析音轨（转音设置变更后）
const reanalyzeTrack = (track: Track) => {
  info(`[RightPanel.vue:77] 开始重新分析音轨${track.id}, 移调${track.transpose}, 转位${track.octave}`);

  // 调试：检查前几个事件的结构
  if (originalMidiEvents.value.length > 0) {
    const sampleEvents = originalMidiEvents.value.slice(0, 5);
    info(`[RightPanel.vue:88] 样本事件: ${JSON.stringify(sampleEvents.map(e => ({ track: e.track, type: e.type, note: e.note })))}`);
  }

  // 获取该音轨的所有原始音符事件（只要note_on事件）
  const trackEvents = originalMidiEvents.value.filter(e => e.track === track.id && e.type === 'note_on');
  info(`[RightPanel.vue:99] 音轨${track.id}的原始note_on事件数量: ${trackEvents.length}, 总原始事件: ${originalMidiEvents.value.length}`);

  if (trackEvents.length === 0) {
    info(`[RightPanel.vue:1010] 音轨${track.id}没有note_on事件，跳过分析`);
    return;
  }

  // 应用转音调整
  const adjustment = track.transpose + (track.octave * 12);
  const adjustedNotes = trackEvents.map(e => e.note + adjustment);

  // 从设置中获取限制值
  const settings = settingsManager.getSettings();
  const limit_min = settings.keySettings?.minNote || 48;
  const limit_max = settings.keySettings?.maxNote || 83;

  // 重新计算分析
  const max_note = Math.max(...adjustedNotes);
  const min_note = Math.min(...adjustedNotes);
  const upper_over_limit = adjustedNotes.filter(n => n > limit_max).length;
  const lower_over_limit = adjustedNotes.filter(n => n < limit_min).length;
  const is_max_over_limit = max_note > limit_max || max_note < limit_min;
  const is_min_over_limit = min_note < limit_min || min_note > limit_max;

  // 重新计算建议
  let suggested_max_transpose = null;
  let suggested_max_octave = null;
  let suggested_min_transpose = null;
  let suggested_min_octave = null;

  if (is_max_over_limit) {
    const diff = limit_max - max_note;
    const suggestion = optimizeTransposeSuggestion(diff, track.transpose, track.octave);
    if (suggestion) {
      suggested_max_transpose = suggestion[0];
      suggested_max_octave = suggestion[1];
    }
  }

  if (is_min_over_limit) {
    const diff = limit_min - min_note;
    const suggestion = optimizeTransposeSuggestion(diff, track.transpose, track.octave);
    if (suggestion) {
      suggested_min_transpose = suggestion[0];
      suggested_min_octave = suggestion[1];
    }
  }

  // 更新音轨分析
  track.analysis = {
    ...track.analysis,
    max_note,
    min_note,
    max_note_name: getNoteName(max_note),
    min_note_name: getNoteName(min_note),
    max_note_group: groupForNote(max_note),
    min_note_group: groupForNote(min_note),
    upper_over_limit,
    lower_over_limit,
    is_max_over_limit,
    is_min_over_limit,
    suggested_max_transpose,
    suggested_max_octave,
    suggested_min_transpose,
    suggested_min_octave,
  };

  info(`[RightPanel.vue:1111] 音轨${track.id}重新分析完成: ${min_note}-${max_note}, 移调${track.transpose}, 转位${track.octave}`);
};

// 优化移调建议（前端版本）
const optimizeTransposeSuggestion = (diff: number, current_transpose: number, current_octave: number): [number, number] | null => {
  const suggestions: Array<[number, number, number]> = [];

  for (let octave_shift = -2; octave_shift <= 2; octave_shift++) {
    const total_transpose_needed = diff - (octave_shift * 12);
    const final_transpose = current_transpose + total_transpose_needed;
    const final_octave = current_octave + octave_shift;

    let score = Math.abs(final_transpose) + Math.abs(final_octave);
    // 鼓励小的移调和转位变化，特别是避免大的移调
    if (Math.abs(final_transpose) >= 5 && Math.abs(final_transpose) <= 7) {
      score += 0.5; // 惩罚大的移调
    }

    suggestions.push([final_transpose, final_octave, score]);
  }

  suggestions.sort((a, b) => a[2] - b[2]);
  return suggestions.length > 0 ? [suggestions[0][0], suggestions[0][1]] : null;
};

// 播放控制
// 播放控制状态
const isPlaying = ref(false);
const playbackRemainingTime = ref(0);
let playbackTimer: number | null = null;
const countdownSeconds = ref(0); // 倒计时秒数
let countdownInterval: number | null = null; // 倒计时定时器
let cancelCountdownFunc: (() => void) | null = null; // 取消倒计时的函数

// 按钮启用状态
const isPlayButtonEnabled = ref(false);
const isStopButtonEnabled = ref(false);
const isPreviewButtonEnabled = ref(false);
const isMidiPlaybackButtonEnabled = ref(false);

// 按钮状态管理函数
const enablePlaybackButtons = () => {
  isPlayButtonEnabled.value = true;
  isStopButtonEnabled.value = false;
  isPreviewButtonEnabled.value = true;
  isMidiPlaybackButtonEnabled.value = true;
};

const setPlayingState = () => {
  isPlayButtonEnabled.value = true; // 播放按钮变为暂停
  isStopButtonEnabled.value = true;
  isPreviewButtonEnabled.value = false;
  isMidiPlaybackButtonEnabled.value = false;
};

const setCountdownState = () => {
  isPlayButtonEnabled.value = false; // 倒计时期间禁用播放按钮
  isStopButtonEnabled.value = true; // 允许取消倒计时
  isPreviewButtonEnabled.value = false;
  isMidiPlaybackButtonEnabled.value = false;
};

const setPreviewState = (isPreviewing: boolean) => {
  isPlayButtonEnabled.value = !isPreviewing;
  isStopButtonEnabled.value = false;
  isPreviewButtonEnabled.value = true; // 预览按钮本身始终启用（用于停止）
  isMidiPlaybackButtonEnabled.value = !isPreviewing;
};

const setMidiPlaybackState = (isPlaying: boolean) => {
  isPlayButtonEnabled.value = !isPlaying;
  isStopButtonEnabled.value = false;
  isPreviewButtonEnabled.value = !isPlaying;
  isMidiPlaybackButtonEnabled.value = true; // 试听按钮本身始终启用（用于停止）
};

// 播放控制
const togglePlay = async () => {
  if (isPlaying.value) {
    // 暂停播放（实际上是停止）
    await stopPlayback();
  } else {
    // 开始播放
    await startPlayback();
  }
};

const startPlayback = async () => {
  if (!props.selectedMidiFile) {
    error('[RightPanel.vue] 没有选择MIDI文件');
    return;
  }

  try {
    info('[RightPanel.vue] 开始准备播放...');

    // 1. 使用 filteredMidiEvents（已经过音轨选择、转音、黑键处理）
    // 2. 过滤超限事件和非 note_on 事件
    const validEvents = filteredMidiEvents.value.filter(event => {
      if (event.type !== 'note_on') return false;
      const note = event.note;
      // 只保留在范围内的音符
      return note >= currentMinNote.value && note <= currentMaxNote.value;
    });

    if (validEvents.length === 0) {
      error('[RightPanel.vue] 没有有效的音符可播放（所有音符都超限或未选中音轨）');
      return;
    }

    info(`[RightPanel.vue] 过滤后有${validEvents.length}个有效音符，原始${filteredMidiEvents.value.filter(e => e.type === 'note_on').length}个音符`);

    // 3. 将 MIDI 音符事件映射到按键事件
    const keyEvents = validEvents.map(event => {
      const key = NOTE_TO_KEY[event.note];
      if (!key) {
        info(`[RightPanel.vue] 警告：音符${event.note}没有对应的按键映射`);
        return null;
      }
      return {
        time: event.time,
        key: key,
        duration: event.duration || 0.1
      };
    }).filter(e => e !== null); // 过滤掉没有映射的音符

    if (keyEvents.length === 0) {
      error('[RightPanel.vue] 没有可映射的按键事件');
      return;
    }

    info(`[RightPanel.vue] 准备播放${keyEvents.length}个按键事件`);

    // 4. 计算总时长
    const maxEndTime = Math.max(...validEvents.map(e => e.end || e.time));
    playbackRemainingTime.value = Math.ceil(maxEndTime);

    // 5. 开始 3 秒倒计时
    // 立即设置倒计时状态，确保停止按钮可用
    setCountdownState();
    countdownSeconds.value = 3;
    info('[RightPanel.vue] 开始倒计时...');

    await new Promise<void>((resolve, reject) => {
      // 定义取消函数
      cancelCountdownFunc = () => {
        if (countdownInterval) {
          clearInterval(countdownInterval);
          countdownInterval = null;
        }
        countdownSeconds.value = 0;
        reject(new Error('Countdown cancelled'));
      };

      countdownInterval = window.setInterval(() => {
        countdownSeconds.value--;
        if (countdownSeconds.value <= 0) {
          if (countdownInterval) {
            clearInterval(countdownInterval);
            countdownInterval = null;
          }
          countdownSeconds.value = 0;
          cancelCountdownFunc = null; // 正常结束，清除取消函数
          resolve();
        }
      }, 1000);
    });

    // 再次确认没有被取消（双重保险）
    if (cancelCountdownFunc === null && countdownSeconds.value > 0) {
      throw new Error('Countdown cancelled');
    }
    cancelCountdownFunc = null; // 清除引用

    // 6. 调用后端 start_playback 命令
    // 在调用后端前，强制设置正确的按钮状态，防止之前的状态混乱
    setPlayingState();

    await invoke('start_playback', { events: keyEvents });
    isPlaying.value = true;
    info('[RightPanel.vue] 播放已启动');

    // 7. 启动播放倒计时
    playbackTimer = window.setInterval(() => {
      playbackRemainingTime.value--;
      if (playbackRemainingTime.value <= 0) {
        stopPlayback();
      }
    }, 1000);

  } catch (e: any) {
    if (e.message === 'Countdown cancelled') {
      info('[RightPanel.vue] 倒计时被取消');
    } else {
      error(`[RightPanel.vue] 播放失败: ${e}`);
    }

    countdownSeconds.value = 0;
    if (countdownInterval) {
      clearInterval(countdownInterval);
      countdownInterval = null;
    }
    cancelCountdownFunc = null;
    await stopPlayback();
  }
};

const stopPlayback = async () => {
  info('[RightPanel.vue] 停止播放');

  // 取消倒计时
  if (cancelCountdownFunc) {
    cancelCountdownFunc();
    cancelCountdownFunc = null;
  }

  if (countdownInterval) {
    clearInterval(countdownInterval);
    countdownInterval = null;
    countdownSeconds.value = 0;
    info('[RightPanel.vue] 倒计时已取消');
  }

  // 调用后端停止播放
  if (isPlaying.value) {
    try {
      await invoke('stop_playback');
    } catch (e) {
      error(`[RightPanel.vue] 停止播放时出错: ${e}`);
    }
  }

  // 清除定时器
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }

  isPlaying.value = false;
  playbackRemainingTime.value = 0;

  // 恢复按钮状态
  enablePlaybackButtons();
};

// 预览功能
const togglePreview = async () => {
  if (isPreviewing.value || isPlayingMidi.value) {
    // 停止预览或播放
    stopMidiPlayback();
  } else {
    // 开始预览
    await startPreview();
  }
};

const startPreview = async () => {
  // 1. 检查是否有选中的MIDI文件
  if (!props.selectedMidiFile) {
    error('[RightPanel.vue] 没有选择MIDI文件');
    return;
  }

  // 2. 过滤有效事件（非超限音符）
  const validEvents = midiEvents.value.filter(event => {
    if (event.type !== 'note_on') return false;
    const note = event.note;
    // 只保留在范围内的音符
    return note >= currentMinNote.value && note <= currentMaxNote.value;
  });

  // 3. 检查是否有有效事件
  if (validEvents.length === 0) {
    error('[RightPanel.vue] 没有有效的音符可预览（所有音符都超限）');
    return;
  }

  info(`[RightPanel.vue] 预览模式：过滤后有${validEvents.length}个有效音符，原始${midiEvents.value.filter(e => e.type === 'note_on').length}个音符`);

  // 4. 临时保存原始事件
  const originalEvents = midiEvents.value;

  // 5. 替换为过滤后的事件
  midiEvents.value = validEvents;

  // 6. 调用现有的播放逻辑
  try {
    await startMidiPlayback();
    isPreviewing.value = true;
    setPreviewState(true); // 设置预览中状态
    info('[RightPanel.vue] 预览播放已启动');
  } catch (e) {
    error(`[RightPanel.vue] 预览播放失败: ${e}`);
    // 恢复原始事件
    midiEvents.value = originalEvents;
    isPreviewing.value = false;
    enablePlaybackButtons();
  }
};

// MIDI播放相关状态
const isPlayingMidi = ref(false);
const isPreviewing = ref(false); // 预览模式状态
const midiRemainingTime = ref(0);
let toneSynth: any = null; // 使用非响应式变量避免 Vue Proxy 干扰 Tone.js

let midiPlaybackTimer: number | null = null;

const toggleMidiPlayback = async () => {
  if (isPlayingMidi.value) {
    // 停止播放
    stopMidiPlayback();
  } else {
    // 开始播放
    setMidiPlaybackState(true); // 设置试听MIDI中状态
    await startMidiPlayback();
  }
};

const startMidiPlayback = async () => {
  if (!props.selectedMidiFile) {
    error('[RightPanel.vue:1212] 没有选择MIDI文件');
    return;
  }

  try {
    info('[RightPanel.vue:1313] 开始加载MIDI播放器...');

    // 动态导入Tone.js
    const Tone = await import('tone');

    // 停止之前的播放
    if (toneSynth) {
      toneSynth.dispose();
    }

    // 创建合成器（使用PolySynth支持多音符同时播放）
    toneSynth = new Tone.PolySynth(Tone.Synth, {
      volume: -5, // 音量95%左右
      oscillator: {
        type: 'triangle'
      },
      envelope: {
        attack: 0.005,
        decay: 0.1,
        sustain: 0.3,
        release: 1
      }
    }).toDestination();

    info('[RightPanel.vue:1414] 开始播放MIDI...');
    isPlayingMidi.value = true;

    // 启动Tone.js音频上下文
    if (Tone.context.state !== 'running') {
      await Tone.start();
    }
    info(`[RightPanel.vue:1515] Tone context state: ${Tone.context.state}`);

    // 计算总时长
    if (midiEvents.value.length === 0) {
      error('[RightPanel.vue:1616] 没有MIDI事件可播放');
      stopMidiPlayback();
      return;
    }

    const maxEndTime = Math.max(...midiEvents.value.map(e => e.end || e.time));
    midiRemainingTime.value = Math.ceil(maxEndTime);

    // 调度所有音符
    const now = Tone.now();
    const startTimeOffset = 0.5; // 延迟0.5秒开始播放，给调度留出时间
    info(`[RightPanel.vue:1717] Current Tone time: ${now}, Scheduling start at: ${now + startTimeOffset}`);
    let scheduledCount = 0;

    midiEvents.value.forEach((event, index) => {
      if (index < 3) {
        info(`[RightPanel.vue:1818] Event ${index}: type=${event.type}, note=${event.note}, time=${event.time}, duration=${event.duration}`);
      }
      if (event.type === 'note_on' && event.note && event.velocity > 0) {
        try {
          // 将MIDI音符号转换为音符名称
          const noteName = Tone.Frequency(event.note, 'midi').toNote();
          const startTime = now + startTimeOffset + event.time;
          const duration = event.duration || 0.5;
          const velocity = Math.min(Math.max(event.velocity / 127, 0.1), 1); // 确保音量在0.1-1之间

          // 调度音符播放
          toneSynth.triggerAttackRelease(
            noteName,
            duration,
            startTime,
            velocity
          );
          scheduledCount++;
        } catch (e) {
          // 忽略单个音符的错误
          error(`[RightPanel.vue:1919] Error scheduling note: ${e}`);
        }
      }
    });

    info(`[RightPanel.vue:2020] 成功调度${scheduledCount}个音符`);

    // 启动倒计时
    midiPlaybackTimer = window.setInterval(() => {
      midiRemainingTime.value--;
      if (midiRemainingTime.value <= 0) {
        stopMidiPlayback();
      }
    }, 1000);

  } catch (e) {
    error(`[RightPanel.vue:2121] MIDI播放失败: ${e}`);
    stopMidiPlayback();
  }
};

const stopMidiPlayback = async () => {
  info('[RightPanel.vue:2222] 停止MIDI播放');

  // 停止合成器
  if (toneSynth) {
    try {
      toneSynth.dispose();
      toneSynth = null;
      info('[RightPanel.vue:2323] 合成器已销毁');
    } catch (e) {
      info(`[RightPanel.vue:2424] 停止合成器时出错: ${e}`);
    }
  }

  // 清除定时器
  if (midiPlaybackTimer) {
    clearInterval(midiPlaybackTimer);
    midiPlaybackTimer = null;
  }
  isPlayingMidi.value = false;
  isPreviewing.value = false; // 重置预览状态
  midiRemainingTime.value = 0;

  // 恢复按钮状态
  enablePlaybackButtons();
};

// const testSound = async () => {
//   try {
//     info('[RightPanel.vue:2525] Testing sound...');
//     const Tone = await import('tone');
//     await Tone.start();
//     info(`[RightPanel.vue:2626] Tone context state: ${Tone.context.state}`);
//     const synth = new Tone.Synth().toDestination();
//     synth.triggerAttackRelease("C4", "8n");
//     info("[RightPanel.vue:2727] Test sound played");
//   } catch (e) {
//     error(`[RightPanel.vue:2828] Test sound failed: ${e}`);
//   }
// };

// 格式化时间显示（MM:SS）
const formatTime = (seconds: number): string => {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

// 对话框显示状态
const showEventTableDialog = ref(false);
const showSettingsDialog = ref(false);
const showHelpDialog = ref(false);

// 显示事件表
const showEventTable = () => {
  showEventTableDialog.value = true;
};

// 显示设置
const showSettings = () => {
  showSettingsDialog.value = true;
};

// 显示帮助
const showHelp = () => {
  showHelpDialog.value = true;
};

// 处理设置保存
const handleSettingsSaved = async (payload: any) => {
  info(`[RightPanel.vue:2929] 设置已保存`);

  // 检查 keySettings 是否变更
  if (payload.keySettingsChanged) {
    info('[RightPanel.vue] keySettings 已变更，更新当前显示范围');

    // 更新当前显示的 min/max note
    const newSettings = settingsManager.getSettings();
    currentMinNote.value = newSettings.keySettings?.minNote || 48;
    currentMaxNote.value = newSettings.keySettings?.maxNote || 83;

    // 如果有选中的 MIDI 文件，重新解析
    if (props.selectedMidiFile) {
      info(`[RightPanel.vue] 检测到 keySettings 变更且有选中文件，重新解析: ${props.selectedMidiFile}`);
      try {
        const result: any = await invoke("parse_midi", {
          filePath: props.selectedMidiFile,
          minNote: currentMinNote.value,
          maxNote: currentMaxNote.value,
          blackKeyMode: newSettings.keySettings?.blackKeyMode || "support_black_key",
          trimLongNotes: newSettings.keySettings?.trimLongNotes || false
        });

        // 更新事件数据
        if (result.events) {
          originalMidiEvents.value = JSON.parse(JSON.stringify(result.events));
          midiEvents.value = result.events;
        }

        // 更新音轨列表
        if (result.tracks) {
          tracks.value = result.tracks.map((t: any) => ({
            id: t.id,
            name: t.name,
            noteCount: t.note_count,
            selected: true,
            transpose: 0,
            octave: 0,
            analysis: t.analysis
          }));
          allTracksSelected.value = true;
        }

        info('[RightPanel.vue] MIDI 文件重新解析完成');
      } catch (e) {
        error(`[RightPanel.vue] 重新解析 MIDI 失败: ${e}`);
      }
    }
  }
};

// 格式化音轨分析文本（返回对象，包含可点击部分）
const getTrackAnalysisLines = (track: Track) => {
  const analysis = track.analysis;
  if (!analysis || !analysis.max_note || !analysis.min_note) {
    return {
      line1: `音符数量: ${track.noteCount}`,
      line2: '',
      suggestions: []
    };
  }

  // 第一行：最高音分析
  const maxStatus = analysis.upper_over_limit > 0 ? "超限" : "未超限";
  const line1 = `最高音: ${analysis.max_note_name}(${analysis.max_note})  ${analysis.max_note_group}  ${maxStatus}  超限数量: ${analysis.upper_over_limit}`;

  // 第二行：最低音分析
  const minStatus = analysis.lower_over_limit > 0 ? "超限" : "未超限";
  const line2 = `最低音: ${analysis.min_note_name}(${analysis.min_note})  ${analysis.min_note_group}  ${minStatus}  超限数量: ${analysis.lower_over_limit}`;

  // 第三行：建议（仅在超限时显示）
  const suggestions: Array<{ type: 'max' | 'min', text: string, transpose: number, octave: number }> = [];

  if (analysis.is_max_over_limit && analysis.suggested_max_transpose !== null && analysis.suggested_max_octave !== null) {
    suggestions.push({
      type: 'max',
      text: '最高音',
      transpose: analysis.suggested_max_transpose,
      octave: analysis.suggested_max_octave
    });
  }

  if (analysis.is_min_over_limit && analysis.suggested_min_transpose !== null && analysis.suggested_min_octave !== null) {
    suggestions.push({
      type: 'min',
      text: '最低音',
      transpose: analysis.suggested_min_transpose,
      octave: analysis.suggested_min_octave
    });
  }

  return { line1, line2, suggestions };
};

// 应用建议
const applySuggestion = (track: Track, type: 'max' | 'min') => {
  const analysis = track.analysis;
  if (!analysis) return;

  info(`[RightPanel.vue:3030] 开始应用建议: 音轨${track.id}, 类型${type}`);

  if (type === 'max' && analysis.suggested_max_transpose !== null && analysis.suggested_max_octave !== null) {
    track.transpose = analysis.suggested_max_transpose;
    track.octave = analysis.suggested_max_octave;
    info(`[RightPanel.vue:3131] 应用最高音建议: 移调${track.transpose}, 转位${track.octave}`);
    reanalyzeTrack(track);
  } else if (type === 'min' && analysis.suggested_min_transpose !== null && analysis.suggested_min_octave !== null) {
    track.transpose = analysis.suggested_min_transpose;
    track.octave = analysis.suggested_min_octave;
    info(`[RightPanel.vue:3232] 应用最低音建议: 移调${track.transpose}, 转位${track.octave}`);
    reanalyzeTrack(track);
  }
};

// 暴露方法给父组件（用于快捷键调用）
defineExpose({
  togglePlay,
  stopPlayback
});

</script>

<template>
  <section class="right-panel">
    <div class="scrollable-content">
      <!-- 音轨详情区域 -->
      <div class="tracks-frame">
        <h3 class="frame-title">
          音轨详情【 当前播放范围：{{ getNoteName(currentMinNote) }}({{ currentMinNote }}) - {{ getNoteName(currentMaxNote) }}({{
            currentMaxNote }}) 】
        </h3>

        <!-- 当前歌曲名称 -->
        <div class="current-song-section">
          <div class="song-info">
            <span class="label">当前歌曲：</span>
            <span class="value">{{ displayFileName || "未选择" }}</span>
          </div>
        </div>

        <!-- 所有音轨复选框 -->
        <div class="all-tracks-section">
          <div class="checkbox-item">
            <input type="checkbox" id="allTracks" v-model="allTracksSelected" @change="toggleSelectAll" />
            <label for="allTracks">所有音轨</label>
          </div>
        </div>

        <!-- 音轨列表 -->
        <div class="tracks-list-section">
          <div class="tracks-list">
            <div v-if="tracks.length === 0" class="empty-tracks-hint">
              {{ props.selectedMidiFile ? "正在解析或无有效音轨..." : "请在左侧选择 MIDI 文件" }}
            </div>
            <div v-else v-for="track in tracks" :key="track.id" class="track-item">
              <!-- 音轨选择 -->
              <div class="track-selection">
                <input type="checkbox" :id="`track-${track.id}`" v-model="track.selected"
                  @change="toggleTrackSelection(track.id)" />
              </div>

              <!-- 音轨信息和分析 -->
              <div class="track-info">
                <div class="track-header">
                  <h4 class="track-title">音轨{{ track.id + 1 }}：{{ track.name }} ({{ track.noteCount }}个音符)</h4>
                </div>
                <div class="track-analysis">
                  <div class="analysis-line">{{ getTrackAnalysisLines(track).line1 }}</div>
                  <div class="analysis-line">{{ getTrackAnalysisLines(track).line2 }}</div>
                  <div v-if="getTrackAnalysisLines(track).suggestions.length > 0" class="analysis-line">
                    建议:
                    <template v-for="(sug, idx) in getTrackAnalysisLines(track).suggestions" :key="idx">
                      <span v-if="idx > 0"> </span>
                      <span class="suggestion-link" @click="applySuggestion(track, sug.type)">{{ sug.text }}</span>
                      <span> 移调{{ sug.transpose }}，转位{{ sug.octave }}</span>
                    </template>
                  </div>
                </div>
              </div>

              <!-- 转音设置 -->
              <div class="transpose-settings">
                <h5 class="transpose-title">转音设置</h5>
                <div class="setting-group">
                  <label>移调:</label>
                  <div class="control-buttons">
                    <button class="btn btn-small" @click="adjustTranspose(track.id, -1)">-</button>
                    <input type="number" v-model.number="track.transpose" class="number-input" />
                    <button class="btn btn-small" @click="adjustTranspose(track.id, 1)">+</button>
                  </div>
                </div>
                <div class="setting-group">
                  <label>转位:</label>
                  <div class="control-buttons">
                    <button class="btn btn-small" @click="adjustOctave(track.id, -1)">-</button>
                    <input type="number" v-model.number="track.octave" class="number-input" />
                    <button class="btn btn-small" @click="adjustOctave(track.id, 1)">+</button>
                  </div>
                </div>
                <div class="reset-link" @click="resetTranspose(track.id)">
                  重置
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作区域 -->
      <div class="controls-frame">
        <h3 class="frame-title">操作</h3>

        <!-- 剩余时间 -->
        <div class="time-section">
          <div class="time-label">剩余时间: {{ isPlayingMidi ? formatTime(midiRemainingTime) : remainingTime }}</div>
        </div>

        <!-- 控制按钮 -->
        <div class="control-buttons-section">
          <button class="btn btn-success" @click="togglePlay" :disabled="!isPlayButtonEnabled">{{ countdownSeconds > 0 ?
            `${countdownSeconds}秒后开始` : (isPlaying ? '暂停模拟' : '开始模拟') }}</button>
          <button class="btn btn-danger" @click="stopPlayback" :disabled="!isStopButtonEnabled">停止模拟</button>
          <button class="btn btn-info" @click="togglePreview" :disabled="!isPreviewButtonEnabled">{{ isPreviewing ?
            '停止试听'
            :
            '效果试听' }}</button>
          <button class="btn btn-info" @click="toggleMidiPlayback" :disabled="!isMidiPlaybackButtonEnabled">{{
            isPlayingMidi ?
              '停止试听' : '文件试听' }}</button>
          <!-- <button class="btn btn-secondary" @click="testSound">测试声音</button> -->
        </div>
      </div>

      <!-- 其他功能 -->
      <div class="other-frame">
        <h3 class="frame-title">其他</h3>

        <!-- 其他按钮 -->
        <div class="other-buttons-section">
          <button class="btn btn-secondary" @click="showEventTable">事件表</button>
          <button class="btn btn-secondary" @click="showSettings">设置</button>
          <button class="btn btn-secondary" @click="showHelp">帮助</button>
        </div>
      </div> <!-- Closing other-frame -->
    </div>
  </section>

  <!-- 事件表对话框 -->
  <EventTableDialog :visible="showEventTableDialog" @update:visible="showEventTableDialog = $event"
    :events="filteredMidiEvents" />
  <SettingsDialog v-model:visible="showSettingsDialog" @settingsSaved="handleSettingsSaved" />
  <HelpDialog v-model:visible="showHelpDialog" />
</template>

<style scoped>
.right-panel {
  flex: 1;
  background-color: var(--bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0rem 0.5rem;
}

/* 通用框架样式 */
.tracks-frame {
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.5rem;
  margin-bottom: 0.4rem;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  min-height: 300px;
}

.controls-frame {
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.5rem;
  margin-bottom: 0.4rem;
}

.other-frame {
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.5rem;
  margin-bottom: 0.2rem;
}

.frame-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0 0 0.5rem 0;
}

/* 音轨详情区域 */
.all-tracks-section {
  margin-bottom: 0.4rem;
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

.current-song-section {
  margin-bottom: 0.4rem;
}

.song-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.song-info .label {
  font-size: 0.9rem;
  color: var(--secondary);
}

.song-info .value {
  font-size: 0.9rem;
  color: var(--fg);
  font-weight: 500;
}

/* 音轨列表 */
.tracks-list-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.5rem;
}

.tracks-list {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.empty-tracks-hint {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: 100px;
  color: var(--secondary);
  font-size: 0.9rem;
}

.track-item {
  display: flex;
  gap: 0.75rem;
  padding: 0.5rem;
  background-color: var(--active);
  border: 1px solid var(--border);
  border-radius: 4px;
  margin-bottom: 0.3rem;
  overflow: hidden;
}

.track-item:last-child {
  margin-bottom: 0;
}

.track-selection {
  display: flex;
  align-items: flex-start;
  padding-top: 0.25rem;
}

.track-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.track-header {
  margin-bottom: 0.25rem;
}

.track-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
}

.track-analysis {
  font-size: 0.8rem;
  flex: 1;
  display: flex;
  flex-direction: column;
  color: var(--fg);
  background-color: var(--bg);
  padding: 0.25rem;
  border-radius: 4px;
  min-width: 0;
  max-height: 100%;
  overflow-y: auto;
}

.analysis-line {
  margin: 0.125rem 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.suggestion-link {
  color: var(--primary);
  cursor: pointer;
  text-decoration: underline;
}

.suggestion-link:hover {
  color: var(--dark);
}

/* 转音设置 */
.transpose-settings {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 150px;
}

.transpose-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0 0 0.25rem 0;
}

.setting-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.setting-group label {
  font-size: 0.8rem;
  color: var(--secondary);
  font-weight: 500;
  min-width: 35px;
}

.control-buttons {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.btn-small {
  padding: 0.25rem 0.5rem;
  font-size: 0.8rem;
  min-width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
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

.number-input {
  width: 50px;
  padding: 0.25rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.8rem;
  text-align: center;
}

.number-input:focus {
  outline: none;
  border-color: var(--primary);
}

.reset-link {
  font-size: 0.8rem;
  color: var(--primary);
  text-decoration: underline;
  cursor: pointer;
  margin-top: 0.25rem;
  align-self: flex-start;
}

.reset-link:hover {
  color: var(--light);
}

/* 操作区域 */
.time-section {
  margin-bottom: 0.4rem;
}

.time-label {
  font-size: 0.9rem;
  color: var(--fg);
  text-align: center;
  font-weight: 500;
}

.control-buttons-section {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

/* 其他功能区域 */
.other-buttons-section {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

/* 按钮样式 */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.4rem 1rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
  max-width: 120px;
}

/* 播放控制按钮 */
.btn-success {
  background-color: var(--success);
  color: var(--selectfg);
  border-color: var(--success);
}

.btn-danger {
  background-color: var(--danger);
  color: var(--selectfg);
  border-color: var(--danger);
}

.btn-info {
  background-color: var(--info);
  color: var(--selectfg);
  border-color: var(--info);
}

/* 其他区域按钮 - 和预览按钮颜色一致 */
.other-buttons-section .btn {
  background-color: var(--info);
  color: var(--selectfg);
  border-color: var(--info);
}

/* 悬停效果 */
.btn:hover {
  background-color: var(--dark);
  border-color: var(--dark);
}

.scrollable-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
</style>
