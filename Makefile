

#windows Configuration
RAM := 8G
CORES := 4
VERSION := win11
PASSWORD := Password123!
USER := docker
CHROME = google-chrome-stable
RDP = xfreerdp3

container_name = avr-simulator-rs-debian
container_name_win = avr-simulator-rs-win
win_build_path = ./target/x86_64-pc-windows-msvc/release
win_data_vol := avr-simulator-rs-win-storage

cargo_test:
	cargo test

cargo_build: sync_repo
	cargo tauri build

sync_repo:
	git submodule update --remote

docker_linux_build: cargo_build
	docker build -t $(container_name) -f Dockerfile-linux .

docker_linux_run:
	docker run -d --rm -it -e PASSWORD=$(PASSWORD) --name $(container_name) -p 3389:3389 -v /tmp/.X11-unix:/tmp/.X11-unix --device /dev/dri:/dev/dri --ipc=host $(container_name)

docker_linux_stop:
	-docker stop $(container_name) -t 10
docker_linux_connect_rdp:
	-xfreerdp3 /v:localhost:3389 /u:$(USER) /p:$(PASSWORD) /cert:ignore /dynamic-resolution +clipboard
	docker stop $(container_name) -t 10

docker_linux: docker_linux_build docker_linux_run docker_linux_connect_rdp
docker_windows_build: sync_repo
	cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
docker_windows_run:
	docker volume create $(win_data_vol)

	docker run -d --rm \
    		--name $(container_name_win) \
    		--device /dev/kvm \
    		--cap-add NET_ADMIN \
    		-p 8006:8006 \
    		-p 3389:3389 \
    		-e VERSION=$(VERSION) \
    		-e RAM_SIZE=$(RAM) \
    		-e CPU_CORES=$(CORES) \
    		-e PASSWORD=$(PASSWORD) \
    		-v $(win_data_vol):/storage \
    		-v $(shell pwd)/target/x86_64-pc-windows-msvc/release:/shared/dist \
    		-v $(shell pwd)/tests:/shared/tests \
    		dockurr/windows
	docker ps

docker_windows_stop:
	-docker stop $(container_name_win) -t 120

docker_windows_connect_rdp:
	xfreerdp3 /v:localhost:3389 /u:$(USER) /p:$(PASSWORD) /cert:ignore /dynamic-resolution +clipboard
	docker stop $(container_name_win) -t 120

docker_windows_connect_web:
	$(CHROME) http://127.0.0.1:8006/

docker_windows: docker_windows_build docker_windows_run docker_windows_connect_rdp
