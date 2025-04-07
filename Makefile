
.POSIX:

build:
	cargo build --color=always --message-format=json-diagnostic-rendered-ansi --package clip_keeper --bin clip_keeper --profile release
	cp ./target/release/clip_keeper ./clip_keeper.AppDir/usr/bin/


appimage:
	mkdir appimage; \
  	appimagetool -s -v clip_keeper.AppDir appimage/clip-keeper.AppImage


install:
	cp appimage/clip-keeper.AppImage /usr/bin/clip-keeper


.PHONY: build, appimage, install