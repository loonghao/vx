# 媒体处理工具

vx 支持用于音频、视频和图像处理的媒体工具。

## FFmpeg

完整的跨平台音视频录制、转换和流媒体解决方案。

```bash
vx install ffmpeg latest

vx ffmpeg -version
vx ffmpeg -i input.mp4 output.avi
vx ffmpeg -i video.mp4 -vn -acodec copy audio.aac
vx ffmpeg -i input.mp4 -vf scale=1280:720 output.mp4
```

**捆绑工具：**
- `ffprobe` - 多媒体流分析器
- `ffplay` - 简易媒体播放器

```bash
# 使用 ffprobe 检查媒体信息
vx ffprobe -v quiet -print_format json -show_format input.mp4

# 使用 ffplay 快速播放
vx ffplay video.mp4
```

## ImageMagick

强大的图像处理和转换工具。

```bash
vx install magick latest

vx magick --version
vx magick input.png output.jpg
vx magick -resize 50% input.png output.png
vx magick montage *.jpg -geometry +2+2 collage.png
```

**平台支持：**
- **Linux**: 通过 AppImage 直接下载
- **macOS**: 通过 Homebrew 安装（`brew install imagemagick`）
- **Windows**: 通过系统包管理器**静默/非交互模式**安装：
  - `winget`（首选，Windows 11 内置）- 使用 `--silent --disable-interactivity`
  - `choco` - 使用 `-y --no-progress --limit-output`
  - `scoop` - 默认非交互

**静默安装：**

在 Windows 上安装 ImageMagick 时，vx 会自动使用静默安装参数以避免交互式提示。这使其适合 CI/CD 环境和自动化工作流程：

```bash
# vx 自动处理所有静默安装参数
vx install magick latest

# 在后台，vx 使用：
# winget: winget install --id ImageMagick.ImageMagick --silent --disable-interactivity
# choco:  choco install imagemagick -y --no-progress --limit-output
```

**常用操作：**

```bash
# 格式转换
vx magick input.png output.jpg

# 调整图片大小
vx magick input.png -resize 800x600 output.png

# 创建缩略图
vx magick input.jpg -thumbnail 150x150^ -gravity center -extent 150x150 thumb.jpg

# 添加水印
vx magick input.png watermark.png -gravity southeast -composite output.png

# 批量转换
vx magick mogrify -format jpg *.png
```

**注意：** 在 ImageMagick 7+ 中，统一的 `magick` 命令取代了旧版的 `convert`、`mogrify` 等命令。请使用 `magick convert` 而不是单独的 `convert`。

## 项目配置示例

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
