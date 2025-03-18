extends Button

func _pressed() -> void:
	$"../PoseSupplier"._reset_timer()
