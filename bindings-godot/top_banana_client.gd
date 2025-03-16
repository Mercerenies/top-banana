extends Node

# TODO Remove this or move it to an examples folder


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
    await $TopBananaClient.submit_score('faf41eee-16f3-4b35-9659-fb80797351d4', "Joe from Godot", 3.1415, "http://godotengine.org").request_completed
    var scores = await $TopBananaClient.get_scores('faf41eee-16f3-4b35-9659-fb80797351d4', 10).request_completed
    for score in scores:
        print(score.player_name + " " + str(score.player_score))
    pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
    pass
