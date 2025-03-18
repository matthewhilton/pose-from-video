extends Node3D

@export var supplier: PoseSupplier
@export var frame_scene: PackedScene
@export var frame_scale: float = 1.0

var frames := Dictionary()

func _ready():
	supplier.new_pose.connect(_on_new_pose)
	supplier.new_frame.connect(_on_new_frame)

func _on_new_frame():
	for frame in frames.values():
		frame.visible = false

func _on_new_pose(marker_id: int, t: Transform3D):
	if !frames.has(marker_id):
		var frame := frame_scene.instantiate()
		frames.set(marker_id, frame)
		add_child(frame)
	
	var frame: CoordinateFrame = frames.get(marker_id)
	frame.global_transform = t.scaled_local(Vector3(frame_scale,frame_scale,frame_scale))
	frame.label.text = str(marker_id)
	frame.visible = true
