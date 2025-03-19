
_server_url = "";

// Hack to get millisecond precision in GM.
_startup_time = 0;
_startup_timestamp = undefined;

_unix_epoch = date_create_datetime(1970, 1, 1, 0, 0, 0);

_empty_map = ds_map_create();

_requests_map = ds_map_create();
