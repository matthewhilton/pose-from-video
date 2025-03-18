use std::{fs, path::{Path, PathBuf}, str::FromStr};
use opencv::prelude::MatTraitConstManual;
use rayon::prelude::*;
use apriltag::{image_buf::DEFAULT_ALIGNMENT_U8, Detector, Family, Image};
use apriltag_video_tracker::{calibration::Calibration, pose::MarkerPose};
use clap::Parser;
use ffmpeg_next::{format::{context::{input::PacketIter, Input}, Pixel}, frame::Video, software::scaling::{Context, Flags}, Rational};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help="Video file to extract poses from")]
    video: PathBuf,
    #[arg(short, long, help="File with calibration parameters stored in TOML format")]
    camera_calibration_file: PathBuf,
    #[arg(short, long, help="File to write output JSON to")]
    out: PathBuf,
    #[arg(short='s', long, default_value="0.011", help="Internal tag marker length in metres")]
    tag_size: f64,
    #[arg(short='t', long, default_value="tagStandard41h12", help="AprilTag family name")]
    tag_family: String
}

#[derive(serde::Serialize)]
struct PosesInFrame {
    t: f64,
    poses: Vec<MarkerPose>
}

struct VideoFrameProducer {
    decoder: ffmpeg_next::codec::decoder::video::Video,
    input: Input,
    time_base: Rational,
    scaler: Context,
    stream_index: usize
}

impl VideoFrameProducer {
    fn new(filename: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        ffmpeg_next::init()?;

        // Find the stream inside the video file to play (files may have multiple streams, not all are video)
        let input = ffmpeg_next::format::input(Path::new(&filename))?;
        let input_stream = input.streams().best(ffmpeg_next::media::Type::Video).ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let time_base =  input_stream.time_base();
        let stream_index = input_stream.index();

        // Setup decoder.
        let context = ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())?;
        let decoder = context.decoder().video()?;

        // Create a scaler to convert frames to RGB
        let scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        Ok(VideoFrameProducer {
            decoder,
            input,
            time_base,
            scaler,
            stream_index
        }) 
    }
}

struct IteratedFrame {
    frame: Video,
    frame_t: f64,
}

impl Iterator for VideoFrameProducer {
    type Item = IteratedFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let mut frame = Video::empty();

        while let Some((stream, packet)) = self.input.packets().next() {
            if stream.index() == self.stream_index {
                if self.decoder.send_packet(&packet).is_ok() {
                    if self.decoder.receive_frame(&mut frame).is_ok() {
                        let frame_t = frame.pts().unwrap_or_default() as f64 * f64::from(self.time_base);

                        // Transcode frame to RGB image.
                        let mut rgb_frame = Video::empty();
                        self.scaler.run(&frame, &mut rgb_frame).unwrap();

                        // Convert RGB frame into a Luma8 image (grayscale).
                        // See https://en.wikipedia.org/wiki/Luma_(video) for formula.
                        let mut luma_data = Vec::with_capacity((self.decoder.width() * self.decoder.height()) as usize);
                        for chunk in rgb_frame.data(0).chunks_exact(3) {
                            let r = chunk[0] as f32;
                            let g = chunk[1] as f32;
                            let b = chunk[2] as f32;
                            let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                            luma_data.push(luma);
                        }
                        return Some(IteratedFrame {
                            frame: rgb_frame,
                            frame_t
                        });
                    }
                }
            }
        }

        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args = Args::parse();

    let calibration: Calibration = toml::from_str(fs::read_to_string(Path::new(&args.camera_calibration_file)).unwrap().as_str()).unwrap();
    let tagparams = calibration.to_params(args.tag_size);
    println!("Got camera calibration: {}", calibration);

    let family = Family::from_str(&args.tag_family).unwrap();
    let mut detector = Detector::builder().add_family_bits(family, 1).build().unwrap();

    let res: Vec<PosesInFrame> = VideoFrameProducer::new(args.video)?.into_iter().map(|f| {
        let width = f.frame.width() as usize;
        let height = f.frame.height() as usize;

        // Convert RGB frame into a Luma8 image (grayscale).
        // See https://en.wikipedia.org/wiki/Luma_(video) for formula.
        let mut luma_data = Vec::with_capacity((width * height) as usize);
        for chunk in f.frame    .data(0).chunks_exact(3) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            luma_data.push(luma);
        }

        // Convert Luma8 into AprilTag image.
        // The Luma8 data is stored in row major, reading left to right, top to bottom.
        let mut image = Image::zeros_with_alignment(width, height, DEFAULT_ALIGNMENT_U8).unwrap();
        for x in 0..width {
            for y in 0..height {
                let idx = y * width + x;
                image[(x, y)] = luma_data[idx];
            }
        }

        // Detect apriltags in image.
        let detections = detector.detect(&image);
        println!("t={:.3}, Found {} apriltags", f.frame_t, detections.len());

        // Calculate their poses.
        let poses: Vec<MarkerPose> = detections.iter().filter_map(|d| MarkerPose::from_detection(d, &tagparams)).collect();

        PosesInFrame {
            t: f.frame_t,
            poses
        }
    }).collect();

    let json_out = serde_json::to_string(&res).unwrap();
    let _ = fs::write(Path::new(&args.out), json_out);

    println!("Done writing output JSON to {}", args.out.to_str().unwrap());

    Ok(())
}
