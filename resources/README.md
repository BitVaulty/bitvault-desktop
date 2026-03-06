# Resources Directory

This directory should contain the following logo files:

## Required Files

1. **app.ico** or **app.png** - The BV logo for the application icon (window title bar)
   - Should be square (e.g., 64x64, 128x128, or 256x256 pixels)
   - Used as the window icon

2. **bitvault_logo.png** or **bitvault_logo.svg** - The larger BitVault logo
   - Used on the PIN entry screen
   - Used on the vault selection screen
   - Should be a larger, more detailed logo suitable for display

## File Locations

The application will look for these files in the following locations (in order):
1. `resources/` (relative to project root when running from repo)
2. `resources/` (relative to the executable)
3. Executable directory + `resources/`

## Supported Formats

- **Icons**: `.ico`, `.png`
- **Logos**: `.png`, `.svg` (SVG support requires egui_extras with svg feature, which is already enabled)
