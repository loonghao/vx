# Media Processing Tools

vx supports media processing tools for audio, video, and image manipulation.

## FFmpeg

A complete, cross-platform solution to record, convert and stream audio and video.

```bash
vx install `ffmpeg@latest

vx ffmpeg -version
vx ffmpeg -i input.mp4 output.avi
vx ffmpeg -i video.mp4 -vn -acodec copy audio.aac
vx ffmpeg -i input.mp4 -vf scale=1280:720 output.mp4
```

**Bundled tools:**
- `ffprobe` - Multimedia stream analyzer
- `ffplay` - Simple media player

```bash
# Use ffprobe for media inspection
vx ffprobe -v quiet -print_format json -show_format input.mp4

# Use ffplay for quick playback
vx ffplay video.mp4
```

## ImageMagick

A powerful image manipulation and conversion tool.

```bash
vx install `magick@latest

vx magick --version
vx magick input.png output.jpg
vx magick -resize 50% input.png output.png
vx magick montage *.jpg -geometry +2+2 collage.png
```

**Platform Support:**
- **Linux**: Direct download via AppImage
- **macOS**: Installed via Homebrew (`brew install imagemagick`)
- **Windows**: Installed via system package managers with **silent/non-interactive mode**:
  - `winget` (preferred, built-in on Windows 11) - Uses `--silent --disable-interactivity`
  - `choco` - Uses `-y --no-progress --limit-output`
  - `scoop` - Non-interactive by default

**Silent Installation:**

When installing ImageMagick on Windows, vx automatically uses silent installation flags to avoid interactive prompts. This makes it suitable for CI/CD environments and automated workflows:

```bash
# vx handles all the silent flags automatically
vx install `magick@latest

# Behind the scenes, vx uses:
# winget: winget install --id ImageMagick.ImageMagick --silent --disable-interactivity
# choco:  choco install imagemagick -y --no-progress --limit-output
```

**Common operations:**

```bash
# Format conversion
vx magick input.png output.jpg

# Resize image
vx magick input.png -resize 800x600 output.png

# Create thumbnail
vx magick input.jpg -thumbnail 150x150^ -gravity center -extent 150x150 thumb.jpg

# Add watermark
vx magick input.png watermark.png -gravity southeast -composite output.png

# Batch convert
vx magick mogrify -format jpg *.png
```

**Note:** In ImageMagick 7+, the unified `magick` command replaces legacy commands like `convert`, `mogrify`, etc. Use `magick convert` instead of just `convert`.

## Project Configuration Example

```toml
[tools]
ffmpeg = "latest"
magick = "latest"

[scripts]
convert-video = "ffmpeg -i input.mp4 -c:v libx264 -preset slow output.mp4"
thumbnail = "magick input.jpg -thumbnail 200x200^ -gravity center -extent 200x200 thumbnail.jpg"
extract-audio = "ffmpeg -i video.mp4 -vn -acodec libmp3lame audio.mp3"
resize-images = "magick mogrify -resize 50% *.jpg"
```
