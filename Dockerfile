FROM debian:latest

WORKDIR /app
RUN apt-get update -y && \
    apt-get install -y --fix-missing \
    libayatana-appindicator3-1 \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    mesa-utils \
    libgl1-mesa-dri \
    libgl1 \
    libglu1-mesa \
    dbus-x11

RUN dbus-uuidgen > /etc/machine-id
COPY target/release/bundle/deb .
COPY tests ./tests
RUN apt-get install ./*.deb -y

ENV NO_AT_BRIDGE=1

CMD ["avr-simulator-rs"]