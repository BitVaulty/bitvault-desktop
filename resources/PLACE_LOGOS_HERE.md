# Place Your Logo Files Here

## Required Files

1. **bitvault_logo.png** (or **bitvault_logo.svg**)
   - The larger BitVault logo for display on screens
   - Will be shown on:
     - PIN entry screen
     - Vault selection screen
   - Recommended size: 400x400px or larger (will be scaled to 200px width)

2. **app.ico** (or **app.png**)
   - The BV logo icon for the window title bar
   - Should be square (e.g., 64x64, 128x128, or 256x256 pixels)
   - Used as the application window icon

## File Location

Place both files directly in this directory:
```
/home/user/src/bitvault-org/bitvault-desktop/resources/
```

## Supported Formats

- **Logos**: `.png`, `.svg` (SVG requires egui_extras with svg feature - already enabled)
- **Icons**: `.ico`, `.png`

## After Placing Files

1. Rebuild the app: `cargo build --release`
2. Run the app - the logos should appear automatically

## Debugging

If logos don't appear, check the console output. The app will print:
- How many locations it's checking
- Which path it found the logo at (if found)
- If no logo is found, it will say "No logo file found in any checked location"
