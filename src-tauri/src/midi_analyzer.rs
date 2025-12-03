use midly::{MidiMessage, Smf, TrackEventKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Black and white key pitch classes (matching Python implementation)
const BLACK_PCS: [u8; 5] = [1, 3, 6, 8, 10]; // C#, D#, F#, G#, A#
const WHITE_PCS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11]; // C, D, E, F, G, A, B

/// Find the nearest white key pitch class for a given pitch class
/// Uses "nearest" strategy: finds the white key with minimum absolute distance
fn nearest_white_pc(pc: u8) -> u8 {
    let pc = pc % 12;

    // If already a white key, return as-is
    if WHITE_PCS.contains(&pc) {
        return pc;
    }

    // Find the nearest white key by minimum distance
    let mut best_pc = 0;
    let mut best_dist = 12;

    for &white_pc in &WHITE_PCS {
        let dist = std::cmp::min(
            (pc as i32 - white_pc as i32).abs(),
            (white_pc as i32 - pc as i32).abs(),
        );
        if dist < best_dist {
            best_dist = dist;
            best_pc = white_pc;
        }
    }

    best_pc
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidiEvent {
    pub time: f64,
    #[serde(rename = "type")]
    pub type_: String, // "note_on" or "note_off"
    pub note: u8,
    pub channel: u8,
    pub track: usize,
    pub velocity: u8,
    pub duration: f64,
    pub end: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    pub min_note: Option<u8>,
    pub max_note: Option<u8>,
    pub under_min_count: usize,
    pub over_max_count: usize,
    pub min_note_name: String,
    pub max_note_name: String,
    pub total_over_limit_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackAnalysis {
    pub max_note: Option<u8>,
    pub min_note: Option<u8>,
    pub max_note_name: String,
    pub min_note_name: String,
    pub max_note_group: String,
    pub min_note_group: String,
    pub upper_over_limit: usize,
    pub lower_over_limit: usize,
    pub is_max_over_limit: bool,
    pub is_min_over_limit: bool,
    pub suggested_max_transpose: Option<i32>,
    pub suggested_max_octave: Option<i32>,
    pub suggested_min_transpose: Option<i32>,
    pub suggested_min_octave: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackInfo {
    pub id: usize,
    pub name: String,
    pub note_count: usize,
    pub analysis: TrackAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidiAnalysis {
    pub events: Vec<MidiEvent>,
    pub analysis: AnalysisResult,
    pub tracks: Vec<TrackInfo>,
}

fn get_note_name(note: u8) -> String {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    let note_index = (note as usize) % 12;
    format!("{}{}", note_names[note_index], octave)
}

fn get_note_group(note: u8) -> String {
    // Based on groups.ts configuration
    match note {
        21..=23 => "大字二组 (A₂-B₂)".to_string(),
        24..=35 => "大字一组 (C₁-B₁)".to_string(),
        36..=47 => "大字组 (C-B)".to_string(),
        48..=59 => "小字组 (c-b)".to_string(),
        60..=71 => "小字一组 (c¹-b¹)".to_string(),
        72..=83 => "小字二组 (c²-b²)".to_string(),
        84..=95 => "小字三组 (c³-b³)".to_string(),
        96..=107 => "小字四组 (c⁴-b⁴)".to_string(),
        108 => "小字五组 (c⁵)".to_string(),
        _ => "未知".to_string(),
    }
}

// 优化移调建议逻辑，以移调+转位的绝对值最小为准，优先选择5、6、7
fn optimize_transpose_suggestion(
    diff: i32,
    current_transpose: i32,
    current_octave: i32,
) -> Option<(i32, i32)> {
    let mut suggestions = Vec::new();

    // 生成所有可能的移调+转位组合（-2到+2个八度）
    for octave_shift in -2..=2 {
        // 计算需要的总移调量
        let total_transpose_needed = diff - (octave_shift * 12);

        // 计算最终的移调和转位值（叠加到当前设置上）
        let final_transpose = current_transpose + total_transpose_needed;
        let final_octave = current_octave + octave_shift;

        // 计算评分：移调+转位的绝对值（越小越好）
        let mut score = final_transpose.abs() as f32 + final_octave.abs() as f32;

        // 如果移调值的绝对值在5、6、7范围内，给予额外加分（优先级更高）
        if (5..=7).contains(&final_transpose.abs()) {
            score -= 0.5; // 给予加分，使这些值优先级更高
        }

        suggestions.push((final_transpose, final_octave, score));
    }

    // 按评分排序（评分越小越优先）
    suggestions.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // 返回最优解
    suggestions.first().map(|(t, o, _)| (*t, *o))
}

pub fn analyze_midi_file(
    file_path: &str,
    min_note: u8,
    max_note: u8,
    black_key_mode: &str,
) -> Result<MidiAnalysis, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let smf = Smf::parse(&bytes).map_err(|e| format!("Failed to parse MIDI: {}", e))?;

    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int() as f64,
        midly::Timing::Timecode(_, _) => return Err("SMPTE timing not supported yet".to_string()),
    };

    let mut events = Vec::new();
    let mut tracks_info = Vec::new();
    let mut tempo_changes = Vec::new(); // (tick, microseconds_per_beat)

    // First pass: collect tempo changes from all tracks (usually track 0)
    // And also track names and per-track note statistics
    let mut track_notes: HashMap<usize, Vec<u8>> = HashMap::new();

    for (i, track) in smf.tracks.iter().enumerate() {
        let mut current_tick = 0;
        let mut track_name = format!("Track {}", i);
        let mut note_count = 0;
        let mut notes_in_track = Vec::new();

        for event in track {
            current_tick += event.delta.as_int();

            match event.kind {
                TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) => {
                    tempo_changes.push((current_tick, t.as_int()));
                }
                TrackEventKind::Meta(midly::MetaMessage::TrackName(name)) => {
                    if let Ok(n) = String::from_utf8(name.to_vec()) {
                        track_name = n;
                    }
                }
                TrackEventKind::Midi {
                    message: MidiMessage::NoteOn { key, vel },
                    ..
                } => {
                    if vel.as_int() > 0 {
                        note_count += 1;
                        notes_in_track.push(key.as_int());
                    }
                }
                _ => {}
            }
        }

        if note_count > 0 {
            // Calculate track analysis using provided min/max note
            let limit_min = min_note;
            let limit_max = max_note;

            let max_note = notes_in_track.iter().max().copied();
            let min_note = notes_in_track.iter().min().copied();

            let upper_over_limit = notes_in_track.iter().filter(|&&n| n > limit_max).count();
            let lower_over_limit = notes_in_track.iter().filter(|&&n| n < limit_min).count();

            let is_max_over_limit = max_note.map_or(false, |n| n > limit_max || n < limit_min);
            let is_min_over_limit = min_note.map_or(false, |n| n < limit_min || n > limit_max);

            // 计算建议值（当前移调和转位都是0）
            let current_transpose = 0;
            let current_octave = 0;

            let (suggested_max_transpose, suggested_max_octave) = if is_max_over_limit {
                max_note
                    .and_then(|n| {
                        let diff = limit_max as i32 - n as i32;
                        optimize_transpose_suggestion(diff, current_transpose, current_octave)
                    })
                    .map(|(t, o)| (Some(t), Some(o)))
                    .unwrap_or((None, None))
            } else {
                (None, None)
            };

            let (suggested_min_transpose, suggested_min_octave) = if is_min_over_limit {
                min_note
                    .and_then(|n| {
                        let diff = limit_min as i32 - n as i32;
                        optimize_transpose_suggestion(diff, current_transpose, current_octave)
                    })
                    .map(|(t, o)| (Some(t), Some(o)))
                    .unwrap_or((None, None))
            } else {
                (None, None)
            };

            let analysis = TrackAnalysis {
                max_note,
                min_note,
                max_note_name: max_note.map(get_note_name).unwrap_or_default(),
                min_note_name: min_note.map(get_note_name).unwrap_or_default(),
                max_note_group: max_note.map(get_note_group).unwrap_or_default(),
                min_note_group: min_note.map(get_note_group).unwrap_or_default(),
                upper_over_limit,
                lower_over_limit,
                is_max_over_limit,
                is_min_over_limit,
                suggested_max_transpose,
                suggested_max_octave,
                suggested_min_transpose,
                suggested_min_octave,
            };

            tracks_info.push(TrackInfo {
                id: i,
                name: track_name,
                note_count,
                analysis,
            });

            track_notes.insert(i, notes_in_track);
        }
    }

    // Sort tempo changes by tick
    tempo_changes.sort_by_key(|k| k.0);
    // Dedup tempo changes (keep last one for same tick)
    let mut unique_tempo_changes: Vec<(u32, u32)> = Vec::new();
    for tc in tempo_changes {
        if let Some(last) = unique_tempo_changes.last_mut() {
            if last.0 == tc.0 {
                *last = tc;
            } else {
                unique_tempo_changes.push(tc);
            }
        } else {
            unique_tempo_changes.push(tc);
        }
    }
    // Ensure there is a tempo at tick 0 (default 120 BPM = 500,000 microseconds per beat)
    if unique_tempo_changes.is_empty() || unique_tempo_changes[0].0 > 0 {
        unique_tempo_changes.insert(0, (0, 500_000));
    }

    // Helper to convert ticks to seconds
    let tick_to_seconds = |tick: u32| -> f64 {
        let mut time = 0.0;
        let mut last_tick = 0;
        let mut last_tempo = 500_000; // Default

        for (t_tick, t_tempo) in &unique_tempo_changes {
            if *t_tick > tick {
                break;
            }
            let delta = *t_tick - last_tick;
            time += (delta as f64 * last_tempo as f64) / (ticks_per_beat * 1_000_000.0);
            last_tick = *t_tick;
            last_tempo = *t_tempo;
        }

        let delta = tick - last_tick;
        time += (delta as f64 * last_tempo as f64) / (ticks_per_beat * 1_000_000.0);
        time
    };

    // Second pass: collect notes
    for (i, track) in smf.tracks.iter().enumerate() {
        let mut current_tick = 0;
        // Key: (channel, note), Value: (start_tick, velocity)
        let mut active_notes: HashMap<(u8, u8), (u32, u8)> = HashMap::new();

        for event in track {
            current_tick += event.delta.as_int();

            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    let channel = channel.as_int();
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int();
                            let velocity = vel.as_int();
                            if velocity > 0 {
                                active_notes.insert((channel, note), (current_tick, velocity));
                            } else {
                                // NoteOn with velocity 0 is NoteOff
                                if let Some((start_tick, start_vel)) =
                                    active_notes.remove(&(channel, note))
                                {
                                    let start_time = tick_to_seconds(start_tick);
                                    let end_time = tick_to_seconds(current_tick);
                                    events.push(MidiEvent {
                                        time: start_time,
                                        type_: "note_on".to_string(),
                                        note,
                                        channel,
                                        track: i,
                                        velocity: start_vel,
                                        duration: end_time - start_time,
                                        end: end_time,
                                    });
                                    events.push(MidiEvent {
                                        time: end_time,
                                        type_: "note_off".to_string(),
                                        note,
                                        channel,
                                        track: i,
                                        velocity: 0,
                                        duration: 0.0,
                                        end: end_time,
                                    });
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int();
                            if let Some((start_tick, start_vel)) =
                                active_notes.remove(&(channel, note))
                            {
                                let start_time = tick_to_seconds(start_tick);
                                let end_time = tick_to_seconds(current_tick);
                                events.push(MidiEvent {
                                    time: start_time,
                                    type_: "note_on".to_string(),
                                    note,
                                    channel,
                                    track: i,
                                    velocity: start_vel,
                                    duration: end_time - start_time,
                                    end: end_time,
                                });
                                events.push(MidiEvent {
                                    time: end_time,
                                    type_: "note_off".to_string(),
                                    note,
                                    channel,
                                    track: i,
                                    velocity: 0,
                                    duration: 0.0,
                                    end: end_time,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Sort events by time
    events.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply black key mode conversion if enabled
    // This matches the Python implementation in midi_analyzer.py lines 529-541
    if black_key_mode == "auto_sharp" {
        for event in &mut events {
            let note = event.note;
            let pc = note % 12;

            // Check if this is a black key
            if BLACK_PCS.contains(&pc) {
                // Find the nearest white key pitch class
                let new_pc = nearest_white_pc(pc);
                // Keep the octave, only change the pitch class
                event.note = (note - pc) + new_pc;
            }
        }
    }

    // Analyze min/max
    let mut min_note = None;
    let mut max_note = None;
    let mut under_min_count = 0;
    let mut over_max_count = 0;

    // Default limits (can be passed as args later)
    let limit_min = 48; // C3? No, 48 is C3 in some standards, C2 in others. Python code says 48.
    let limit_max = 83; // B5?

    for event in &events {
        if event.type_ == "note_on" {
            if min_note.is_none() || event.note < min_note.unwrap() {
                min_note = Some(event.note);
            }
            if max_note.is_none() || event.note > max_note.unwrap() {
                max_note = Some(event.note);
            }

            if event.note < limit_min {
                under_min_count += 1;
            }
            if event.note > limit_max {
                over_max_count += 1;
            }
        }
    }

    // Debug: print first few events
    if events.len() > 0 {
        println!("First 3 events:");
        for (i, event) in events.iter().take(3).enumerate() {
            println!("Event {}: {:?}", i, event);
        }
    }

    Ok(MidiAnalysis {
        events,
        analysis: AnalysisResult {
            min_note,
            max_note,
            under_min_count,
            over_max_count,
            min_note_name: min_note.map(get_note_name).unwrap_or_default(),
            max_note_name: max_note.map(get_note_name).unwrap_or_default(),
            total_over_limit_count: under_min_count + over_max_count,
        },
        tracks: tracks_info,
    })
}
