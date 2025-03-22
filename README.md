
# TopBanana

A highscore engine for video games.

## Setup (Server)

To host an instance of the TopBanana server, you will need:

* [PostgreSQL 14 or newer](https://www.postgresql.org/download/)
* [Rust 1.84 or newer](https://www.rust-lang.org/tools/install)
* [Diesel CLI](https://diesel.rs/guides/getting-started.html)

Additionally, these instructions are geared toward Linux installs. If
you're on Windows and aren't comfortable with WSL or Cygwin, you may
have to adjust some steps.

Ensure that the above tools are installed and that a PostgreSQL server
is running. Then set up a PostgreSQL database and a user for the
TopBanana server to use. Default recommendation is to call both the
database and user `topbanana`, but any name will do.

```
postgres=# CREATE USER topbanana WITH PASSWORD 'xxx'; -- Generate a good password
postgres=# CREATE DATABASE topbanana OWNER topbanana;
```

Once PostgreSQL is set up, you'll need to build your environment.
Create a `.env` file containing the three environment variables
(`template.env` can be used as a starting point for this):

* `DATABASE_URL` shall be the resource identifier for the PostgreSQL database.
* `ROCKET_DATABASES` can be left at its default value in `template.env`.
* `JWT_SECRET_KEY` should be set to a long, unique string of random bytes.

Source the environment and build the server.

```
source .env
cd topbanana-backend/
cargo build
```

Now run the Diesel migrations to setup the PostgreSQL tables.

```
# In topbanana-backend/
diesel migration run
```

If you get an error about 'peer'-authentication, ensure that you have
set up a password-based authentication scheme for PostgreSQL. See
[`pg_hba.conf`](https://www.postgresql.org/docs/current/auth-pg-hba-conf.html)
for more details.

To set up the initial admin user account, run `cargo run --
--generate-initial-user`. Save this user's API key, as you'll need it
to access the API.

Finally, run the server.

```
# In topbanana-backend/
cargo run
```

The server runs on port 8000 by default. This can be overridden with
the `ROCKET_PORT` environment variable. If you go to
`http://localhost:8000/`, you will see a simple welcome page.
`http://localhost:8000/swagger-ui/` contains more detailed API
capabilities.

## Developer API

The API documentation is available at `/swagger-ui/`. Note that the
endpoints under `/api/` are intended to be used by *developers*
creating a game. The endpoints that should be accessed by a *game* are
different.

To receive a JWT token for your user, POST to `/api/authorize/` with
the `X-Api-Key` header containing your API key. All other API
endpoints expect `Authorization: Bearer <your-jwt-token>` as
authentication. The API is where you may create new games and new
highscore tables for existing games.

## Language Bindings

There are currently two language bindings available for TopBanana:

* [Godot 4+](bindings-godot/)
* [Game Maker: Studio 2.3+](bindings-gamemaker/)

If you wish to use TopBanana with a language or engine not listed
here, see [Game API](#game-api) below.

IMPORTANT NOTE: When you create a game, you will be asked to set that
game's *security level*. The default value of 10 is suitable for most
applications, as this requires that your game use modern up-to-date
hashing algorithms. However, Game Maker does not support any modern
hashing algorithms, so if you intend to use the Game Maker bindings,
you must lower the security value to 0.

## Game API

The following endpoints are available to video games wishing to view
or modify highscore tables. All endpoints take JSON as the request
body, but see below for details on how to encode the request.

* `GET /tables/scores` takes `table_uuid`
* `GET /tables/scores?limit=<limit>` takes `table_uuid`
* `POST /tables/scores/new` takes `table_uuid`, `player_name`,
  `player_score`, and optionally `player_score_metadata`.

Highscores are always sorted from highest to lowest floating-point
value, so if you have a table where the lowest score should be in
first place (such as a time trial or a golf game), use negative
numbers. The `player_score_metadata` optional field can be an
arbitrary string and is not used directly by the engine. It can be
used to store information about the player's run that led to this
score, both for visualization purposes or for anti-cheat purposes.

In addition to the parameters listed above, every JSON request object
shall include the following fields:
* `game_uuid` - The UUID of the relevant game.
* `request_uuid` - A unique identifier generated just for this
  request. Client-side code is responsible for generating this. It
  must be a UUID (any version will do) and must only be used once.
* `request_timestamp` - When this request was initiated, as a number
  of seconds since the Unix epoch.
* `algo` - The hashing algorithm used to sign this request. Valid
  options are `sha1` and `sha256`. `sha1` can only be used if the
  game's security level is 0 or below (see the note above in Language
  Bindings).

Once the JSON request object has been constructed, the client must
base64-encode it. Next, join the base64-encoded JSON request with the
game's secret key via a dot, to get
`<json-request-base64>.<secret-key>`. Hash this string with the chosen
hashing algorithm and base64-encode the result. Now send, as the
message body, the following string.

```
<json-request-base64>.<hash-base64>
```

The hash, request UUID, game UUID, and timestamp will all be verified
on the server side, and an HTTP 403 will be issued if any of them are
incorrect or inconsistent.

## License

Available under the [MIT License](LICENSE)
