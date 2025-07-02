
_server_url = "";

// Hack to get millisecond precision in GM.
_startup_time = 0;
_startup_timestamp = undefined;

// We want the Unix epoch. But in some timezones, Game Maker will end
// up constructing a value before the Unix epoch, which is a hard
// error in GM. So instead, we get 2am-ish the day after, to be safe.
// The server only cares that this value is within +/- 48 hours of the
// Unix epoch.
_unix_epoch = date_create_datetime(1970, 1, 2, 2, 1, 5);

_empty_map = ds_map_create();

_requests_map = ds_map_create();
