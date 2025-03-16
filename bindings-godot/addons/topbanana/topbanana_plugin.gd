@tool
extends EditorPlugin

const TopBananaClient = preload("res://addons/topbanana/topbanana_client.gd")


func _enter_tree() -> void:
    add_custom_type("TopBananaClient", "Node", TopBananaClient, null)


func _exit_tree() -> void:
    remove_custom_type("TopBananaClient")
