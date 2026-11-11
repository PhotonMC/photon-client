//! What is read from and written to the connection and to files, and the data tables built from them.

use crate::version::Version;

rules! {
    /// The rules of the wire and of files (see the module).
    ProtocolRules {
        /// A negative element count in a tag list is refused, because the list is sized from the
        /// count (26.3; before it the loop simply reads nothing).
        negative_tag_counts_fail: since V26_3,
        /// The game has the improved transparency option and its debug key (new in 26.3).
        has_improved_transparency: since V26_3,
        /// An enum read by ordinal takes the first constant when the id is out of range (26.3's
        /// `OutOfBoundsStrategy.ZERO`; before it the read fails).
        out_of_range_enum_ids_read_as_first: since V26_3,
        /// The spruce leaves exist as the tinted-particle class of the older versions.
        spruce_leaves_are_tinted: before V26_3,
        /// A level particles packet names its particle first, gives a top speed for each axis and
        /// its count as a variable-length number, and ends with how the spread is drawn (26.3);
        /// 26.2 has the flags first, one top speed, a four-byte count and the particle last.
        level_particles_carry_axis_speeds: since V26_3,
        /// An explosion packet ends with whether its sound plays (26.3); before it the sound
        /// always played.
        explosion_packet_has_sound_flag: since V26_3,
        /// A game mode in the spawn info is a variable-length number, and the previous one is
        /// that number plus one with zero for none (26.3); before it each was one signed byte,
        /// with minus one for no previous mode.
        spawn_info_game_modes_are_var_ints: since V26_3,
    }
}
