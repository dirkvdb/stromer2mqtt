build_debug:
    cargo build

build_release:
    cargo build --release

build: build_release

test_debug test_name='' $RUST_LOG="debug":
    cargo nextest run --no-capture {{ test_name }}

test_release test_name='':
    cargo nextest run --release {{ test_name }}

test: test_release

prod:
    cargo run --release --  --mqtt-addr=192.168.1.13 --mqtt-base-topic home/stromer

dev $RUST_LOG="stromer2mqtt=debug" $S2M_MQTT_USER="iot" $S2M_MQTT_PASS=`cat /run/secrets/mqtt_pass 2>/dev/null || echo ""`:
    cargo run --release -- -vv --mqtt-addr=192.168.1.13 --mqtt-base-topic dbg/home/stromer

docker-nix:
    nix build .#dockerImage
    docker load < result

docker:
    docker build -t dirkvdb/stromer2mqtt:latest -f docker/BuildDockerfile .

dockerdev:
    docker build -t dirkvdb/stromer2mqtt:develop -f docker/BuildDockerfile .

dockerup:
    docker push dirkvdb/stromer2mqtt:latest

dockerdevup:
    docker push dirkvdb/stromer2mqtt:develop
