use std::fmt::{Display, Formatter, Result as FmtResult};
use apriltag::TagParams;
use serde::{Deserialize, Serialize};
use opencv::prelude::*;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Calibration {
    focal_x_pixels: f64,
    focal_y_pixels: f64,
    principal_point_x_pixels: f64,
    principal_point_y_pixels: f64
}

impl Display for Calibration {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "(fx: {}, fy:{}, px: {}, py: {})", self.focal_x_pixels, self.focal_y_pixels, self.principal_point_x_pixels, self.principal_point_y_pixels)
    }
}

impl From<Mat> for Calibration {
    fn from(matrix: Mat) -> Self {
        Calibration {
            focal_x_pixels: matrix.at_2d::<f64>(0,0).unwrap().clone(),
            focal_y_pixels: matrix.at_2d::<f64>(1,1).unwrap().clone(),
            principal_point_x_pixels: matrix.at_2d::<f64>(0,2).unwrap().clone(),
            principal_point_y_pixels: matrix.at_2d::<f64>(1,2).unwrap().clone()
        }
    }
}

impl Calibration {
    pub fn to_params(&self, tagsize: f64) -> TagParams {
        TagParams {
            tagsize,
            fx: self.focal_x_pixels,
            fy: self.focal_y_pixels,
            cx: self.principal_point_x_pixels,
            cy: self.principal_point_y_pixels
        }
    }
}