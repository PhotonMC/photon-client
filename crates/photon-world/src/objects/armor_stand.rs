//! What an armor stand keeps for its drawing: its flags and the six
//! rotations its parts are posed from (`ArmorStand.DATA_CLIENT_FLAGS` and
//! the `DATA_*_POSE` accessors).

use crate::constants::*;
use crate::metadata::{find_metadata_field, metadata_byte_value};

/// `ArmorStand.DEFAULT_*_POSE`, in degrees about x, y, z.
const DEFAULT_POSES: [[f32; 3]; 6] = [
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [-10.0, 0.0, -10.0],
    [-15.0, 0.0, 10.0],
    [-1.0, 0.0, -1.0],
    [1.0, 0.0, 1.0],
];

/// The rotations of an armor stand's parts, in the order the data defines
/// them: the head, the body, the left arm, the right arm, the left leg, the
/// right leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmorStandState {
    pub(crate) flags: u8,
    pub(crate) poses: [[u32; 3]; 6],
}

impl Default for ArmorStandState {
    fn default() -> Self {
        Self {
            flags: 0,
            poses: DEFAULT_POSES.map(|pose| pose.map(f32::to_bits)),
        }
    }
}

impl ArmorStandState {
    /// `ArmorStand.isSmall`.
    pub fn is_small(&self) -> bool {
        self.flags & 1 != 0
    }

    /// `ArmorStand.showArms`.
    pub fn shows_arms(&self) -> bool {
        self.flags & 4 != 0
    }

    /// `ArmorStand.showBasePlate`: it is on unless the stand is told to lose it.
    pub fn shows_base_plate(&self) -> bool {
        self.flags & 8 == 0
    }

    /// `ArmorStand.isMarker`.
    pub fn is_marker(&self) -> bool {
        self.flags & 16 != 0
    }

    /// The rotation of a part, in degrees: 0 head, 1 body, 2 left arm, 3 right
    /// arm, 4 left leg, 5 right leg.
    pub fn pose(&self, part: usize) -> [f32; 3] {
        self.poses[part].map(f32::from_bits)
    }

    /// What a data packet changes: the flags and each rotation it names.
    pub(crate) fn apply(&mut self, metadata: &[u8]) {
        if let Some(flags) = metadata_byte_value(metadata, ARMOR_STAND_FLAGS_DATA_ID) {
            self.flags = flags;
        }
        for (index, pose) in self.poses.iter_mut().enumerate() {
            let data_id = ARMOR_STAND_HEAD_POSE_DATA_ID + index as u8;
            if let Some((ROTATIONS_SERIALIZER_ID, value)) = find_metadata_field(metadata, data_id)
                && let Some(bytes) = value.get(..12)
            {
                for (axis, chunk) in pose.iter_mut().zip(bytes.as_chunks::<4>().0.iter()) {
                    *axis = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_data_packet_changes_what_it_names_and_leaves_the_rest() {
        let mut stand = ArmorStandState::default();
        assert!(stand.shows_base_plate() && !stand.shows_arms());
        let mut data = vec![ARMOR_STAND_FLAGS_DATA_ID, 0, 0b101];
        data.extend([
            ARMOR_STAND_HEAD_POSE_DATA_ID + 1,
            ROTATIONS_SERIALIZER_ID as u8,
        ]);
        for angle in [1.5_f32, -2.0, 30.0] {
            data.extend(angle.to_be_bytes());
        }
        data.push(255);
        stand.apply(&data);
        assert!(stand.is_small() && stand.shows_arms() && !stand.is_marker());
        assert_eq!(stand.pose(1), [1.5, -2.0, 30.0]);
        assert_eq!(stand.pose(3), [-15.0, 0.0, 10.0]);
    }
}
