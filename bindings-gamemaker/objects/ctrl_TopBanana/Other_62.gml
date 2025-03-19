
var request_id = async_load[? "id"];
if (ds_map_exists(_requests_map, request_id)) {
  var callback = _requests_map[? request_id];

  var status = async_load[? "status"];
  if (status < 0) {
    ds_map_delete(_requests_map, request_id);
    throw "Error occurred on high score tables";
  } else if (status == 1) {
    // Downloading; do nothing for now
  } else {
    ds_map_delete(_requests_map, request_id);
    var http_status = async_load[? "http_status"];
    if ((http_status < 200) || (http_status > 299)) {
      throw "Error occurred on high score tables";
    }
    callback();
  }
}
