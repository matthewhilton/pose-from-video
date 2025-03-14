use apriltag::{Pose, TagParams, Detection};

#[derive(serde::Serialize)]
pub struct MarkerPose {
    marker_id: usize,
    pose: SerialisablePose
}

impl MarkerPose {
    pub fn from_detection(detection: &Detection, params: &TagParams) -> Option<Self> {
        let pose = detection.estimate_tag_pose(params)?;
        Some(MarkerPose {
            marker_id: detection.id(),
            pose: SerialisablePose::from(pose)
        })
    }
}

#[derive(serde::Serialize)]
struct SerialisablePose {
    rotation: Vec<f64>,
    translation: Vec<f64>
}

impl From<Pose> for SerialisablePose {
    fn from(value: Pose) -> Self {
        SerialisablePose {
            rotation: value.rotation().data().to_vec(),
            translation: value.translation().data().to_vec()
        }
    }
}