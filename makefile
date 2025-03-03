.PHONY: dev build ui android

dev:
	cd bitvault-app && cargo tauri dev

build:
	cd bitvault-app && cargo tauri build

ui:
	cd bitvault-ui && cargo leptos watch

trunk:
	cd bitvault-ui && trunk build

android:
	cd bitvault-app && cargo tauri android dev