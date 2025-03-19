
function TopBanana_init() {
  if (instance_exists(ctrl_TopBanana)) {
    throw "TopBanana already initialized; you should only call TopBanana_init once!";
  }
  var inst = instance_create_depth(0, 0, 0, ctrl_TopBanana);
  inst._startup_time = current_time;
  inst._startup_timestamp = date_current_datetime();
}

function TopBanana_set_server_url(url) {
  ctrl_TopBanana._server_url = url;
}

function _TopBanana_Game(game_uuid_, game_secret_key_) constructor {
  _game_uuid = game_uuid_;
  _game_secret_key = game_secret_key_;
}

function TopBanana_new_game(game_uuid_, game_secret_key_) {
  return new _TopBanana_Game(game_uuid_, game_secret_key_);
}

function TopBanana_get_scores(game, table_uuid, limit, callback) {
  var url = ctrl_TopBanana._server_url + "tables/scores"
  if (!is_undefined(limit)) {
    url = url + "?limit=" + string(limit);
  }
  var payload = _TopBanana_make_auth_payload(game, {
    "table_uuid": table_uuid,
  })
  var payload_json = json_stringify(payload);
  var payload_base64 = _TopBanana_base64url(payload_json);
  var signature = _TopBanana_sha1(game, payload_base64);

  var body = payload_base64 + "." + signature;
  var request_id = http_request(url, "GET", ctrl_TopBanana._empty_map, body);
  ctrl_TopBanana._requests_map[? request_id] = callback;
}

function _TopBanana_make_auth_payload(game, struct) {
  struct.game_uuid = game._game_uuid;
  struct.request_uuid = _TopBanana_make_uuidv7();
  struct.request_timestamp = int64(date_second_span(ctrl_TopBanana._unix_epoch, date_current_datetime()));
  struct.algo = "sha1";
  return struct;
}

function _TopBanana_make_uuidv7() {
  var millis_since_epoch = _TopBanana_get_millis_since_epoch();
  var uuid = "";

  // Time bits
  uuid += _TopBanana_int_to_hex((millis_since_epoch >> 16) & 0xFFFFFFFF, 8);
  uuid += "-";
  uuid += _TopBanana_int_to_hex(millis_since_epoch & 0xFFFF, 4);

  // Version bits
  uuid += "-7";
  var version_nibble = 0x80 & irandom(63);
  uuid += _TopBanana_int_to_hex(version_nibble, 1);

  // Random bits
  uuid += _TopBanana_int_to_hex(irandom(255), 2);
  uuid += "-";
  uuid += _TopBanana_int_to_hex(irandom(65535), 4);
  uuid += "-";
  uuid += _TopBanana_int_to_hex(irandom(16777215), 6);
  uuid += _TopBanana_int_to_hex(irandom(16777215), 6);

  return uuid;
}

function _TopBanana_get_millis_since_epoch() {
  var millis_since_startup = current_time - ctrl_TopBanana._startup_time;
  var now = date_inc_second(ctrl_TopBanana._startup_timestamp, floor(millis_since_startup / 1000));
  var now_millis = millis_since_startup % 1000;
  var seconds_since_epoch = date_second_span(ctrl_TopBanana._unix_epoch, now);
  return seconds_since_epoch * 1000 + now_millis;
}

function _TopBanana_int_to_hex(n, padded_to) {
  var result = "";
  while (n != 0) {
    result = _TopBanana_to_hex_digit(n & 0x0F) + result;
    n = floor(n / 16);
  }

  if (!is_undefined(padded_to)) {
    while (string_length(result) < padded_to) {
      result = "0" + result;
    }
  }

  return result;
}

function _TopBanana_to_hex_digit(digit) {
  if (digit <= 9) {
    return chr(ord("0") + digit);
  } else {
    return chr(ord("a") + (digit - 10));
  }
}

function _TopBanana_from_hex_digit(digit) {
  var n = ord(digit);
  if (n >= ord("0") && n <= ord("9")) {
    return n - ord("0");
  } else {
    return n - ord("a") + 10;
  }
}

function _TopBanana_base64url(text) {
  var out = base64_encode(text);
  out = string_replace_all(out, "+", "-");
  out = string_replace_all(out, "/", "_");
  return out;
}

function _TopBanana_sha1(game, text_base64) {
  var full_payload = text_base64 + "." + game._game_secret_key;
  var hex_hash = sha1_string_utf8(full_payload);

  // Convert to byte buffer
  var buf = buffer_create(40, buffer_fixed, 1);
  for (var i = 0; i < 20; i++) {
    var ch1 = _TopBanana_from_hex_digit(string_char_at(hex_hash, i * 2 + 1));
    var ch2 = _TopBanana_from_hex_digit(string_char_at(hex_hash, i * 2 + 2));
    var n = ch1 * 16 + ch2;
    buffer_write(buf, buffer_u8, n);
  }
  var hash_base64 = buffer_base64_encode(buf, 0, buffer_get_size(buf));
  buffer_delete(buf);

  hash_base64 = string_replace_all(hash_base64, "+", "-");
  hash_base64 = string_replace_all(hash_base64, "/", "_");
  return hash_base64;
}
