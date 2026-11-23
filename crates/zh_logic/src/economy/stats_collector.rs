//! `PKT-124`: Player command and unit telemetry collector
//! (`GeneralsMD/Code/GameEngine/Source/Common/StatsCollector.cpp`).

use crate::economy::money::LOGICFRAMES_PER_SECOND;
use crate::gamelogic::message_stream::GameMessageType;
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_common::types::{Bool, Int, UnsignedInt, FALSE, TRUE};

/// Output sink abstraction for [`StatsCollector`], replacing direct C++ `FILE*` I/O
/// so callers can log deterministically in memory or to a chosen writer.
pub trait StatsOutputSink {
    fn open_new(&mut self, file_name: &str);
    fn append_text(&mut self, file_name: &str, text: &str);
}

/// In-memory [`StatsOutputSink`] capturing all deterministic log lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryStatsSink {
    pub file_name: String,
    pub contents: String,
}

impl StatsOutputSink for MemoryStatsSink {
    fn open_new(&mut self, file_name: &str) {
        self.file_name = file_name.to_string();
        self.contents.clear();
    }

    fn append_text(&mut self, _file_name: &str, text: &str) {
        self.contents.push_str(text);
    }
}

/// No-op [`StatsOutputSink`] when file telemetry is not needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopStatsSink;

impl StatsOutputSink for NoopStatsSink {
    fn open_new(&mut self, _file_name: &str) {}
    fn append_text(&mut self, _file_name: &str, _text: &str) {}
}

/// World-object descriptor read by [`StatsCollector::collect_unit_count_stats`].
#[derive(Debug, Clone)]
pub struct StatsObjectInfo<'a> {
    pub kind_of: &'a KindOfMaskType,
    pub is_neutral_controlled: bool,
    pub controlling_side: &'a str,
    pub is_local_player: bool,
}

/// Context snapshot supplied to [`StatsCollector`] lifecycle methods.
#[derive(Debug, Clone, Copy)]
pub struct StatsContext<'a> {
    pub frame: UnsignedInt,
    pub play_stats: Int,
    pub map_name: &'a str,
    pub local_player_side: &'a str,
    pub local_player_money: UnsignedInt,
}

/// Player command, scroll-time, and unit-count telemetry collector (`StatsCollector.h/.cpp`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatsCollector {
    pub stats_file_name: String,
    pub build_commands: UnsignedInt,
    pub move_commands: UnsignedInt,
    pub attack_commands: UnsignedInt,
    pub scroll_map_commands: UnsignedInt,
    pub ai_units: UnsignedInt,
    pub player_units: UnsignedInt,
    pub scroll_begin_time: UnsignedInt,
    pub scroll_time: UnsignedInt,
    pub is_scrolling: Bool,
    pub time_count: Int,
    pub last_update: Int,
    pub start_frame: Int,
}

impl StatsCollector {
    pub fn new(current_frame: UnsignedInt) -> Self {
        Self {
            stats_file_name: String::new(),
            build_commands: 0,
            move_commands: 0,
            attack_commands: 0,
            scroll_map_commands: 0,
            ai_units: 0,
            player_units: 0,
            scroll_begin_time: 0,
            scroll_time: 0,
            is_scrolling: FALSE,
            time_count: 0,
            last_update: 0,
            start_frame: current_frame as Int,
        }
    }

    pub fn reset(
        &mut self,
        ctx: &StatsContext<'_>,
        objects: &[StatsObjectInfo<'_>],
        date_suffix: &str,
        sink: &mut dyn StatsOutputSink,
    ) {
        self.create_file_name(ctx.map_name, date_suffix);
        self.write_initial_file_info(ctx, objects, sink);
        self.zero_out_stats();
        self.last_update = ctx.frame as Int;
    }

    pub fn collect_msg_stats(
        &mut self,
        local_player_index: Int,
        msg_player_index: Int,
        msg_type: GameMessageType,
    ) {
        if local_player_index != msg_player_index {
            return;
        }
        match msg_type {
            GameMessageType::MsgQueueUnitCreate
            | GameMessageType::MsgDozerConstruct
            | GameMessageType::MsgDozerConstructLine => {
                self.build_commands = self.build_commands.wrapping_add(1);
            }
            _ => {}
        }
    }

    pub fn collect_unit_count_stats(&mut self, objects: &[StatsObjectInfo<'_>]) {
        for obj in objects {
            let is_unit = obj.kind_of.test(KindOfType::Infantry as Int) != FALSE
                || obj.kind_of.test(KindOfType::Vehicle as Int) != FALSE;
            if !is_unit || obj.is_neutral_controlled || obj.controlling_side == "Civilian" {
                continue;
            }
            if obj.is_local_player {
                self.player_units = self.player_units.wrapping_add(1);
            } else {
                self.ai_units = self.ai_units.wrapping_add(1);
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &StatsContext<'_>,
        objects: &[StatsObjectInfo<'_>],
        sink: &mut dyn StatsOutputSink,
    ) {
        let interval = ctx.play_stats.wrapping_mul(LOGICFRAMES_PER_SECOND as Int);
        let threshold = self.last_update.wrapping_add(interval) as UnsignedInt;
        if threshold > ctx.frame {
            return;
        }

        self.collect_unit_count_stats(objects);

        if self.is_scrolling != FALSE {
            self.scroll_time = self
                .scroll_time
                .wrapping_add(ctx.frame.wrapping_sub(self.scroll_begin_time));
            self.scroll_begin_time = ctx.frame;
        }

        self.time_count = self.time_count.wrapping_add(ctx.play_stats);
        self.write_stat_info(ctx.local_player_money, sink);
        self.zero_out_stats();
        self.last_update = ctx.frame as Int;
    }

    #[inline]
    pub fn increment_scroll_move_count(&mut self) {
        self.scroll_map_commands = self.scroll_map_commands.wrapping_add(1);
    }

    #[inline]
    pub fn increment_attack_count(&mut self) {
        self.attack_commands = self.attack_commands.wrapping_add(1);
    }

    #[inline]
    pub fn increment_build_count(&mut self) {
        self.build_commands = self.build_commands.wrapping_add(1);
    }

    #[inline]
    pub fn increment_move_count(&mut self) {
        self.move_commands = self.move_commands.wrapping_add(1);
    }

    #[inline]
    pub fn start_scroll_time(&mut self, frame: UnsignedInt) {
        self.is_scrolling = TRUE;
        self.scroll_begin_time = frame;
        self.scroll_map_commands = self.scroll_map_commands.wrapping_add(1);
    }

    #[inline]
    pub fn end_scroll_time(&mut self, frame: UnsignedInt) {
        if self.is_scrolling == FALSE {
            return;
        }
        self.is_scrolling = FALSE;
        self.scroll_time = self
            .scroll_time
            .wrapping_add(frame.wrapping_sub(self.scroll_begin_time));
    }

    pub fn write_file_end(&mut self, ctx: &StatsContext<'_>, sink: &mut dyn StatsOutputSink) {
        let elapsed_frames = ctx.frame.wrapping_sub(self.last_update as UnsignedInt);
        let elapsed_secs = (elapsed_frames / (LOGICFRAMES_PER_SECOND as UnsignedInt)) as Int;
        self.time_count = self.time_count.wrapping_add(elapsed_secs);
        self.write_stat_info(ctx.local_player_money, sink);

        let footer = format!(
            "---------------------------------------------------\n\n\
             =KEY===============================================\n\
             Time* = The Time Interval\n\
             BC = Build Commands\n\
             MC = Move Commands\n\
             AC = Attack Commands\n\
             SMC = Scroll Map Commands\n\
             ST* = Scroll Time in Seconds\n\
             OC = Other Commands (N/A)\n\
             $$$ = Local Player's Cash Amount\n\
             #PU = # of Player's Units\n\
             #AIU = # of AI's Units\n\
             ===================================================\n\
             * Times are in Game Seconds which are based off of frames. Current fps is set to {LOGICFRAMES_PER_SECOND}\n"
        );
        sink.append_text(&self.stats_file_name, &footer);
    }

    pub fn zero_out_stats(&mut self) {
        self.build_commands = 0;
        self.move_commands = 0;
        self.attack_commands = 0;
        self.scroll_map_commands = 0;
        self.ai_units = 0;
        self.player_units = 0;
        self.scroll_time = 0;
    }

    fn create_file_name(&mut self, map_name: &str, date_suffix: &str) {
        let file_part = map_name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(map_name);
        let stem = if file_part.len() >= 4 {
            &file_part[..file_part.len() - 4]
        } else {
            ""
        };
        self.stats_file_name = format!("Stats\\{stem}{date_suffix}.txt");
    }

    fn write_initial_file_info(
        &mut self,
        ctx: &StatsContext<'_>,
        objects: &[StatsObjectInfo<'_>],
        sink: &mut dyn StatsOutputSink,
    ) {
        sink.open_new(&self.stats_file_name);
        self.collect_unit_count_stats(objects);
        let header = format!(
            "---------------------------------------------------\n\
             Map:\t{}\n\
             Side:\t{}\n\
             ---------------------------------------------------\n\n\
             Time*\tBC\tMC\tAC\tSMC\tST*\tOC\t$$$\t#PU\t#AIU\n\
             0\t{}\t{}\t{}\t{}\t0\t0\t{}\t{}\t{}\n",
            ctx.map_name,
            ctx.local_player_side,
            self.build_commands,
            self.move_commands,
            self.attack_commands,
            self.scroll_map_commands,
            ctx.local_player_money,
            self.player_units,
            self.ai_units
        );
        sink.append_text(&self.stats_file_name, &header);
    }

    fn write_stat_info(&self, local_player_money: UnsignedInt, sink: &mut dyn StatsOutputSink) {
        let scroll_secs = self.scroll_time / (LOGICFRAMES_PER_SECOND as UnsignedInt);
        let row = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t0\t{}\t{}\t{}\n",
            self.time_count,
            self.build_commands,
            self.move_commands,
            self.attack_commands,
            self.scroll_map_commands,
            scroll_secs,
            local_player_money,
            self.player_units,
            self.ai_units
        );
        sink.append_text(&self.stats_file_name, &row);
    }
}
