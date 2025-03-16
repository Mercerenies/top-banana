extends Node

class Request:
    signal request_completed(payload)

const HighscoreEntry = preload("highscore_entry.gd")

@export var server_url: String
@export var game_uuid: String
@export var game_secret_key: String

var _crypto = Crypto.new()


func get_scores(table_uuid: String, limit = null):
    var url := server_url + "tables/scores"
    if limit != null:
        url = url + "?limit=" + str(limit)
    var payload := _make_auth_payload({ "table_uuid": table_uuid })
    var payload_json := JSON.stringify(payload)
    var payload_base64 := _base64url(payload_json)
    var signature := _get_sha256_signature(payload_base64)

    var http_request := HTTPRequest.new()
    add_child(http_request)
    var request_obj := Request.new()
    http_request.request_completed.connect((func _then(result, resp_code, headers, body):
            http_request.queue_free()
            if result != 0:
                push_error("Error %s: %s" % [result, body.get_string_from_utf8()])
                return
            if resp_code < 200 || resp_code > 299:
                push_error("Error %s: %s" % [resp_code, body.get_string_from_utf8()])
                return
            var json := JSON.parse_string(body.get_string_from_utf8())
            if json == null:
                push_error("Error: Invalid JSON")
                return
            var highscore_entries = json['scores'].map(_create_highscore_entry)
            request_obj.request_completed.emit(highscore_entries)),
        CONNECT_ONE_SHOT)
    http_request.request(url, [], HTTPClient.METHOD_GET, payload_base64 + "." + signature)
    return request_obj


func submit_score(table_uuid: String, name: String, score: float, metadata = null):
    var url := server_url + "tables/scores/new"
    var payload := _make_auth_payload({
        "table_uuid": table_uuid,
        "player_name": name,
        "player_score": score,
        "player_score_metadata": metadata,
    })
    var payload_json := JSON.stringify(payload)
    var payload_base64 := _base64url(payload_json)
    var signature := _get_sha256_signature(payload_base64)

    var http_request := HTTPRequest.new()
    add_child(http_request)
    var request_obj := Request.new()
    http_request.request_completed.connect((func _then(result, resp_code, headers, body):
            http_request.queue_free()
            if result != 0:
                push_error("Error %s: %s" % [result, body.get_string_from_utf8()])
                return
            if resp_code < 200 || resp_code > 299:
                push_error("Error %s: %s" % [resp_code, body.get_string_from_utf8()])
                return
            request_obj.request_completed.emit(null)),
        CONNECT_ONE_SHOT)
    http_request.request(url, [], HTTPClient.METHOD_POST, payload_base64 + "." + signature)
    return request_obj


func _create_highscore_entry(json: Dictionary):
    return HighscoreEntry.new(json["player_name"], json["player_score"], json["player_score_metadata"], json["creation_timestamp"])


func _make_auth_payload(kwargs: Dictionary) -> Dictionary:
    kwargs["game_uuid"] = game_uuid
    kwargs["request_uuid"] = _make_uuidv4()
    kwargs["request_timestamp"] = int(Time.get_unix_time_from_system())
    kwargs["algo"] = "sha256"
    return kwargs


func _make_uuidv4() -> String:
    var bytes := _crypto.generate_random_bytes(16)

    # Set the UUID version (4) and variant (RFC 4122)
    bytes[6] = (bytes[6] & 0x0F) | 0x40
    bytes[8] = (bytes[8] & 0x3F) | 0x80

    var hex := bytes.hex_encode()
    return "%s-%s-%s-%s-%s" % [
        hex.substr(0, 8),
        hex.substr(8, 4),
        hex.substr(12, 4),
        hex.substr(16, 4),
        hex.substr(20, 12)
    ]


func _base64url(data: String) -> String:
    var data64 := Marshalls.utf8_to_base64(data)
    return data64.replace("+", "-").replace("/", "_")


func _base64url_bytes(data: PackedByteArray) -> String:
    var data64 := Marshalls.raw_to_base64(data)
    return data64.replace("+", "-").replace("/", "_")


func _get_sha256_signature(payload_base64: String) -> String:
    var full_payload := "%s.%s" % [payload_base64, game_secret_key]
    var ctx := HashingContext.new()
    ctx.start(HashingContext.HASH_SHA256)
    ctx.update(full_payload.to_utf8_buffer())
    var res = ctx.finish()
    return _base64url_bytes(res)
