# ASCII Art Generator

The ASCII Art Generator is a fun feature that converts images and text into ASCII art, perfect for the Corgi theme of CorgiTerm!

## Features

### 1. Image to ASCII Art
- Load any image file (PNG, JPEG, GIF, etc.)
- Adjustable width (20-200 characters)
- Multiple character sets:
  - **Simple**: ` .:-=+*#@`
  - **Detailed**: Full 70-character gradient
  - **Blocks**: Unicode block characters `░▒▓█`
  - **Braille**: High-detail braille patterns
- ANSI color support
- Invert mode for white backgrounds
- Automatic aspect ratio correction
- Live preview

### 2. Text to ASCII Art
- Convert text to figlet-style ASCII art
- Two built-in fonts:
  - Standard (6-line height)
  - Small (5-line height)
- Perfect for banners and headers
- Max 20 characters (ASCII art gets wide!)

### 3. Built-in Corgi Collection
Six pre-made ASCII art corgis:
- **Classic**: The iconic running corgi
- **Sitting**: Cute sitting pose
- **Running**: Energetic corgi in motion
- **Happy**: Happy corgi face
- **Sleepy**: Sleepy corgi with zzZ
- **Pixel**: CorgiTerm's NES-style tri-color mascot!

Plus a "Random Corgi" button for fun!

## Usage

### Via Menu
1. Open CorgiTerm
2. Click the menu button (⋮)
3. Select **Tools → ASCII Art Generator**

### The Dialog
Three tabs:
- **From Image**: Convert images to ASCII
- **From Text**: Create ASCII text banners
- **🐕 Corgi Art**: Browse the corgi collection

### Buttons
- **Copy**: Copy to clipboard
- **Insert**: Insert into terminal (coming soon)
- **Close**: Close the dialog

## Technical Details

### Image Processing
- Uses the `image` crate for loading/processing
- Brightness mapping: Converts pixels to characters based on perceived brightness
- Weighted RGB: `brightness = 0.299*R + 0.587*G + 0.114*B` (human eye sensitivity)
- ANSI 256-color support for colored output
- Lanczos3 filtering for high-quality resizing

### Character Sets
Character density increases from left to right:
- Simple is great for low-detail images
- Detailed works well for photos
- Blocks give a retro feel
- Braille is best for high-detail art

### Aspect Ratio
Terminal characters are approximately 2:1 (height:width), so the generator applies a 0.5 aspect ratio correction by default to prevent vertically-stretched images.

## Examples

### Convert an Image
```bash
# Load an image and adjust settings:
1. Click "Choose Image..."
2. Select your image
3. Adjust width slider (default: 80 chars)
4. Choose character set
5. Toggle colored mode if desired
6. Click "Copy" or "Insert"
```

### Create a Text Banner
```bash
# Type some text (max 20 chars)
1. Switch to "From Text" tab
2. Enter text: "CORGI"
3. Select font (Standard/Small)
4. Preview updates automatically
5. Click "Copy" or "Insert"
```

### Insert a Corgi
```bash
# Choose from the collection
1. Switch to "🐕 Corgi Art" tab
2. Select from dropdown or click "Random Corgi"
3. Preview shows the art
4. Click "Insert" to add to terminal
```

## Code Structure

### Core (`corgiterm-core/src/ascii_art.rs`)
- `AsciiArtGenerator`: Main generator engine
- `CharacterSet`: Different ASCII character palettes
- `AsciiFont`: Figlet-style text fonts
- `CorgiArt`: Built-in corgi collection

### UI (`corgiterm-ui/src/ascii_art_dialog.rs`)
- `AsciiArtDialog`: GTK4 dialog with three tabs
- Image tab: File picker + settings + preview
- Text tab: Input + font selector + preview
- Corgi tab: Selection + random button + preview

### Integration
- Menu: Tools → ASCII Art Generator
- Keyboard shortcut: (Coming soon)
- Terminal insertion: (Coming soon)

## Future Enhancements

1. **More Fonts**: Additional figlet fonts
2. **Animation**: Animated ASCII art from GIFs
3. **Terminal Insertion**: Direct insertion into terminal
4. **Custom Corgis**: User-submitted corgi art
5. **Color Schemes**: Custom ANSI color palettes
6. **Export**: Save to .txt file
7. **Templates**: Pre-configured settings for common use cases
8. **Keyboard Shortcut**: Quick access via Ctrl+Shift+A

## Sample Corgi

Here's Pixel, the CorgiTerm mascot:

```
    ████████
  ██░░██░░██
  ██░░██░░██  PIXEL
  ██████████  CorgiTerm Mascot
██░░██████░░██
██░░░░░░░░░░██
██░░██░░██░░██
  ████  ████
  ██      ██
  ▓▓      ▓▓

NES-style Tri-Color Corgi!
```

Enjoy making ASCII art! 🐕✨
