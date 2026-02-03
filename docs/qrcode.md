# QR Code Generator

Generate QR codes from URLs or text with optional customization including logo overlay, background images, and AI-generated styling.

## Usage

```bash
dev-swiss qrcode <CONTENT> [OPTIONS]
```

## Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--format` | `-f` | Output format: `terminal`, `png`, `svg` | terminal |
| `--output` | `-o` | Output file path (required for png/svg) | - |
| `--error-correction` | `-e` | Error correction: `low`, `medium`, `quartile`, `high` | medium |
| `--scale` | `-s` | Pixels per QR module for image output | 8 |
| `--invert` | | Invert colors (swap dark/light) | false |
| `--no-quiet-zone` | | Remove border around QR code | false |
| `--logo` | | Path to logo image for center overlay | - |
| `--logo-size` | | Logo size as % of QR code (5-30) | 20 |
| `--background` | | Path to background image | - |
| `--dark-color` | | Dark module color (hex or name) | black |
| `--light-color` | | Light module color (hex or name) | white |
| `--ai-prompt` | | AI prompt for artistic styling (requires API key) | - |
| `--api-key` | | Stability AI API key | env: STABILITY_API_KEY |

## Examples

### Basic Terminal Output

```bash
dev-swiss qrcode "https://example.com"
```

Output:
```
    █▀▀▀▀▀█ ▀▄█ █▄█ ▄ █▀▀▀▀▀█
    █ ███ █ ▀█▀ ▄█▄ ▀ █ ███ █
    █ ▀▀▀ █ ▄█ █▄ █   █ ▀▀▀ █
    ▀▀▀▀▀▀▀ ▀▄▀ █ █▄█ ▀▀▀▀▀▀▀
    ...
```

### Inverted Terminal Output

```bash
dev-swiss qrcode "https://example.com" --invert
```

### Save as PNG

```bash
dev-swiss qrcode "https://example.com" -f png -o qr.png
```

### Save as SVG

```bash
dev-swiss qrcode "https://example.com" -f svg -o qr.svg
```

### Custom Colors

```bash
dev-swiss qrcode "https://example.com" -f png -o qr.png \
  --dark-color "#1a1a2e" --light-color "#eaeaea"
```

Supported color formats:
- Named colors: `black`, `white`, `red`, `green`, `blue`
- Hex colors: `#ff5500` or `ff5500`

### With Logo Overlay

```bash
dev-swiss qrcode "https://example.com" -f png -o qr.png \
  --logo company-logo.png --logo-size 25
```

Note: Logo overlay automatically uses high error correction to ensure scannability.

### With Background Image

```bash
dev-swiss qrcode "https://example.com" -f png -o qr.png \
  --background texture.jpg
```

### AI-Generated Styling

Requires Stability AI API key:

```bash
export STABILITY_API_KEY="your-api-key"
dev-swiss qrcode "https://example.com" -f png -o qr.png \
  --ai-prompt "watercolor art style with soft colors"
```

Or provide key directly:

```bash
dev-swiss qrcode "https://example.com" -f png -o qr.png \
  --ai-prompt "cyberpunk neon" --api-key "sk-..."
```

## Error Correction Levels

| Level | Recovery | Best For |
|-------|----------|----------|
| `low` | ~7% | Clean environments, maximum data |
| `medium` | ~15% | General use (default) |
| `quartile` | ~25% | Slightly damaged codes |
| `high` | ~30% | Logo overlay, damaged codes |

## Tips

1. **Logo overlay**: Use `--logo-size` between 15-25% for best results. The tool auto-selects high error correction.

2. **Printing**: Use `--scale 16` or higher for print-quality PNGs.

3. **Dark backgrounds**: Use `--invert` or swap colors with `--dark-color white --light-color black`.

4. **Testing**: Always scan generated QR codes with a phone to verify they work, especially after adding logos.

## Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Content too large | Data exceeds QR capacity | Shorten URL or use lower error correction |
| Logo too large | Logo size outside 5-30% | Adjust `--logo-size` |
| Invalid color | Unrecognized color format | Use hex (#RRGGBB) or named color |
| Background too small | Background smaller than QR | Use larger background image |
