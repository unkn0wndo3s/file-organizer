# Debian and Ubuntu packages

## Build requirements

```bash
sudo apt install build-essential dpkg-dev curl pkg-config \
     libx11-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
     libfontconfig1-dev libfreetype6-dev libvulkan-dev libasound2-dev

# Rust, if it is not installed already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building

```bash
./packaging/debian/build-deb.sh
```

The package is written to `target/debian/file-organizer_<version>_<arch>.deb`.

## Installing

```bash
sudo apt install ./target/debian/file-organizer_*.deb
```

`apt` pulls the runtime dependencies listed in `control.in`. Installing with
`dpkg -i` instead works too, but then missing dependencies have to be resolved
by hand with `sudo apt --fix-broken install`.

## Starting with the session

The package deliberately does not enable autostart for every user on the
machine. Each user turns it on for themselves:

```bash
file-organizer --enable-autostart
```

or ticks **Start with the session** in the tray menu.

## Removing

```bash
sudo apt remove file-organizer
```

The autostart entry lives in each user's own `~/.config/autostart`, so remove
it before uninstalling with `file-organizer --disable-autostart`.
