container_name = avr-simulator-rs-debian


sync_repo:
	git submodule update --remote

docker_build:
	docker build -t $(container_name) .

docker_run:
	docker run $(container_name) -it -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix --device /dev/dri:/dev/dri --ipc=host
