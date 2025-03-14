🚧 Under Construction

# Apriltag Video
Takes in a video, and extracts Apriltag pose information from each frame.

Additionally, includes a camera calibration script.

# Usage

## Calibration:
1. Print out the OpenCV calibration pattern: https://github.com/opencv/opencv/blob/4.x/doc/pattern.png
2. Place on a flat well lit surface, and take lots of photos with the camera you want to calibrate.
3. Copy the JPG files to a folder, and pass in the folder using `--images-path xxxx`:

`cargo run --bin calibration -- --images-path calibration --output-file camera.toml`

4. This will output a `.toml` file which can be passed in to the command below to properly calculate the pose of april tags.

## Detecting April Tags in video frames
1. Record a video with the camera as calibrated above, that has Apriltags in it.
2. Run the following:

`cargo run --bin parse -- --video VIDEO_FILE --camera-calibration-file CALIBRATION_FILE --out poses.json`

3. This will examine every frame of the video and output the poses of each marker in each frame.