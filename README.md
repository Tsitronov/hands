# hands

Offline Tauri app (Rust + React) that helps adults with mild intellectual
disability learn daily routines through a simple game. The first routine is
handwashing.

## How it works

- Six picture steps; the player picks the next step out of three options.
- Four help levels: video, highlighted, pictures only, independent.
- Two clean laps in a row move the player to the next level.
- Each lap is saved locally (date, name, level, errors, time) and can be
  exported to Excel.

## Notes

- Works fully offline. All data stays on the device (SQLite).
- It is an assistive tool, not a medical device and not a replacement for the
  work of educators.
- Do not publish exported files: they contain names and results.

## Build

    npm install
    npm run tauri dev
    npm run tauri build

## License

MIT