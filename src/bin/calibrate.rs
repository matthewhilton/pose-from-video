use std::fs::{self, DirEntry};
use std::iter::repeat_with;
use std::path::Path;
use apriltag_video_tracker::calibration::Calibration;

use clap::Parser;
use opencv::core::{Point2f, Point3f, Size, TermCriteria, TermCriteria_EPS, TermCriteria_MAX_ITER, Vector};
use opencv::prelude::*;
use opencv::{imgcodecs, imgproc};
use opencv::calib3d::{calibrate_camera_def, find_chessboard_corners_def};

// Default is the standard pattern from https://github.com/opencv/opencv/blob/4.x/doc/pattern.png
const CHESSBOARD_WIDTH: i32 = 6;
const CHESSBOARD_HEIGHT: i32 = 9;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "calibration", help="Folder containing .JPGs of chessboard calibration patterns")]
    images_path: String,
    #[arg(short, long, default_value = "camera.toml", help="File to store calibration parameters in TOML format")]
    output_file: String,
}

fn main() {
    let args = Args::parse();
    let calibration = calibrate_from_images(Path::new(&args.images_path));
    let _ = fs::write(Path::new(&args.output_file), toml::to_string(&calibration).unwrap());
}

// Adapted from example https://github.com/twistedfall/opencv-rust/blob/master/examples/camera_calibration.rs
fn calibrate_from_images(dir: &Path) -> Calibration {
    // Termination criteria, i.e. how far to minmimise errors before it is "good enough".
	let criteria = TermCriteria {
		typ: TermCriteria_EPS + TermCriteria_MAX_ITER,
		max_count: 30,
		epsilon: 0.001,
	};

    // Read all the images, filtering only for jpgs.
    let images: Vec<DirEntry> = fs::read_dir(dir).unwrap()
		.flatten()
		.filter(|entry| entry.path().extension().is_some_and(|ext| ext.to_ascii_lowercase() == "jpg"))
        .collect();

    println!("Found {} calibration images", images.len());
    
    let mut image_points = Vector::<Vector<Point2f>>::new();
    let pattern_size = Size::new(CHESSBOARD_WIDTH, CHESSBOARD_HEIGHT);
    let mut img_size = Size::default();

    for image in images {
        // Load and convert to grayscale.
        let img = imgcodecs::imread_def(image.path().to_string_lossy().as_ref()).unwrap();
		let mut gray = Mat::default();
		imgproc::cvt_color_def(&img, &mut gray, imgproc::COLOR_BGR2GRAY).unwrap();
        img_size = gray.size().unwrap();

        // Find corners.
        let mut corners = Vector::<Point2f>::default();
        let corners_found = find_chessboard_corners_def(&gray, pattern_size, &mut corners).unwrap();

        if corners_found {
            // Refine the corners.
            imgproc::corner_sub_pix(&gray, &mut corners, Size::new(11, 11), Size::new(-1, -1), criteria).unwrap();

            // Push the corners.
            image_points.push(corners);
        }
    }

    // Define 3D points for a single standard chessboard. I.e. each square's (x,y,z) position. Z is a flat plane.
    let objp_len = CHESSBOARD_WIDTH * CHESSBOARD_HEIGHT;
	let objp = Vector::from_iter((0..objp_len).map(|i| Point3f::new((i % CHESSBOARD_WIDTH) as f32, (i / CHESSBOARD_WIDTH) as f32, 0.)));

    // Make N copies of the checkerboard objects.
    let object_points: Vector::<Vector<Point3f>> = repeat_with(|| objp.clone()).take(image_points.len()).collect();

    // Perform the calibration.
    // This essentially tries to find how the 2D corner points translate to the 3D chessboard points.
    let mut mtx = Mat::default();
    let mut dist = Mat::default();
    let mut rvecs = Vector::<Mat>::new();
    let mut tvecs = Vector::<Mat>::new();
    let _ = calibrate_camera_def(
        &object_points,
        &image_points,
        img_size,
        &mut mtx,
        &mut dist,
        &mut rvecs,
        &mut tvecs
    );

    Calibration::from(mtx)
}

