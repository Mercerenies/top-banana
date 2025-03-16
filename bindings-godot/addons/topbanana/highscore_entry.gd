extends RefCounted

var player_name: String
var player_score: float
var player_score_metadata
var creation_timestamp: String


func _init(
        _player_name: String,
        _player_score: float,
        _player_score_metadata,
        _creation_timestamp: String
) -> void:
    self.player_name = _player_name
    self.player_score = _player_score
    self.player_score_metadata = _player_score_metadata
    self.creation_timestamp = _creation_timestamp
