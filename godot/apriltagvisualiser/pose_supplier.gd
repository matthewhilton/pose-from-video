class_name PoseSupplier extends Node

@export_file("*.json") var pose_file

signal new_frame
signal new_pose(marker_id: int, pose: Transform3D)

var _internal_frame_data: Array[FrameData] = []

var _internal_buffer: Array[FrameData] = []
var _current_t: float = 0.0

class FrameData:
	var t: float
	var frame_idx: int
	var marker_poses: Array[MarkerPose] = []
	
	static func parse_from_entry(dict: Dictionary) -> FrameData:
		var data := FrameData.new()
		data.t = dict.get("t")
		data.frame_idx = dict.get("frame_idx")
		data.marker_poses.assign(dict.get("poses").map(func(p): return MarkerPose.parse_from_entry(p)))
		return data

class MarkerPose:
	var marker_id: int
	var pose: Transform3D
	
	static func parse_from_entry(dict: Dictionary) -> MarkerPose:
		var marker_pose := MarkerPose.new()
		marker_pose.marker_id = dict.get("marker_id")
		
		var t = dict.get("pose").get("translation")
		var translation := Vector3(t[0], t[1], t[2])
		
		var r = dict.get("pose").get("rotation")
		var basis := Basis()
		
		# Convert row major input to Godot's column major
		basis.x = Vector3(r[0], r[3], r[6])
		basis.y = Vector3(r[1], r[4], r[7])
		basis.z = Vector3(r[2], r[5], r[8])
		
		marker_pose.pose = Transform3D(basis, translation)
		
		# Convert Z+ of AprilTag to Y+ of Godot.
		marker_pose.pose = marker_pose.pose.rotated(Vector3.RIGHT, deg_to_rad(90))
		
		return marker_pose
	
func _ready() -> void:
	# Read input and parse it into Godot classes.
	var poses_str = FileAccess.open(pose_file, FileAccess.READ).get_as_text()
	var parsed = JSON.parse_string(poses_str)
	_internal_frame_data.assign(parsed.map(func(p): return FrameData.parse_from_entry(p)))
	_reset_timer()

func _process(delta: float) -> void:
	_current_t += delta
	
	# Find any that are now in the past, and emit them and remove from buffer.
	var frames_to_emit: Array[FrameData] = _internal_buffer.filter(func(f): return _current_t > f.t)
	for frame in frames_to_emit:
		print("Emitting frame: ", frame.frame_idx, " t: ", frame.t)
		new_frame.emit()
		for pose in frame.marker_poses:
			new_pose.emit(pose.marker_id, pose.pose)
		
		_internal_buffer.erase(frame)

func _reset_timer():
	_internal_buffer = _internal_frame_data.duplicate()
	_current_t = 0.0
