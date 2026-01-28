# VX Promotional Video

A Remotion-based promotional video showcasing the VX universal development tool manager.

## Project Structure

```
vx-promo-video/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── Background.tsx  # Scene backgrounds
│   │   ├── Title.tsx      # Animated title component
│   │   ├── FeatureCard.tsx # Feature showcase cards
│   │   └── CodeBlock.tsx   # Code block display
│   ├── scenes/              # Video scenes
│   │   ├── IntroScene.tsx        # Introduction
│   │   ├── ProblemScene.tsx        # Problem statement
│   │   ├── SolutionScene.tsx       # Solution showcase
│   │   ├── FeaturesScene.tsx        # Key features
│   │   └── CallToActionScene.tsx    # CTA
│   └── Root.tsx             # Root composition
├── package.json
├── tsconfig.json
└── remotion.config.ts
```

## Video Scenes

### 1. Introduction (4 seconds)
- Displays "🚀 VX - Universal Development Tool Manager"
- Smooth fade-in animation
- Dark background

### 2. Problem (10 seconds)
- Shows the pain points of managing multiple tools
- Displays multiple commands (npx, uvx, go, cargo)
- Highlights complexity and inconsistency

### 3. Solution (10 seconds)
- Introduces VX as the solution
- Shows simplified commands with 'vx' prefix
- Green theme representing solution

### 4. Features (12 seconds)
- Four key feature cards:
  - ⚡ Zero Learning Curve
  - 🌐 Multi-Language Support
  - 🤖 AI-Native Design
  - 🔧 Auto-Install
- Purple theme

### 5. Call to Action (6 seconds)
- Installation commands
- GitHub repository link
- Blue theme

## Getting Started

### Install Dependencies

```bash
cd vx-promo-video
npm install
```

### Preview in Browser

```bash
npm start
```

### Render Video

```bash
npm run build
```

### Render Specific Scene

```bash
# Render only the intro scene
npx remotion render src/Root.tsx vx-intro --out=out/intro.mp4

# Render only features
npx remotion render src/Root.tsx vx-features --out=out/features.mp4
```

## Customization

### Modify Colors

Edit component files to change color schemes:

```tsx
// Background.tsx
<Background color="#0f172a" variant="dark" />

// Title.tsx
<Title color="#ffffff" />
```

### Adjust Timing

Modify `durationInFrames` in `Root.tsx`:

```tsx
<Composition
  id="vx-intro"
  component={IntroScene}
  durationInFrames={120}  // 4 seconds at 30fps
  fps={30}
  width={1920}
  height={1080}
/>
```

### Add New Scenes

1. Create scene in `src/scenes/`
2. Import in `Root.tsx`
3. Add composition definition

## Video Specifications

- **Resolution**: 1920x1080 (Full HD)
- **Frame Rate**: 30 fps
- **Total Duration**: ~42 seconds
- **Format**: MP4 (H.264)

## Components

### Background
- Three variants: dark, light, gradient
- Easy color customization

### Title
- Animated fade-in and slide-up
- Support for subtitle
- Configurable font size and color

### FeatureCard
- Glass morphism design
- Spring animation
- Icon, title, description

### CodeBlock
- Syntax highlighting style
- Fade-in animation
- Language label

## Advanced Features

### Adding Transitions

Install remotion/transitions:

```bash
npm install @remotion/transitions
```

```tsx
import {TransitionSeries} from '@remotion/transitions';

<TransitionSeries>
  <TransitionSeries.Sequence name="first" durationInFrames={60}>
    <FirstScene />
  </TransitionSeries.Sequence>
  <TransitionSeries.Transition in={<FirstScene />} out={<SecondScene />}>
    {/* Custom transition */}
  </TransitionSeries.Transition>
</TransitionSeries>
```

### Audio Integration

```tsx
import {Audio} from 'remotion';

<AbsoluteFill>
  <Audio src="/path/to/music.mp3" />
  <YourContent />
</AbsoluteFill>
```

### Server-Side Rendering

```ts
import {bundle} from '@remotion/bundler';
import {renderMedia} from '@remotion/renderer';

const bundleLocation = await bundle({
  entryPoint: './src/Root.tsx',
});

await renderMedia({
  composition,
  serveUrl: bundleLocation,
  codec: 'h264',
  outputLocation: 'output/video.mp4',
});
```

## Troubleshooting

### Rendering Issues

1. Ensure Chrome/Chromium is installed
2. Check for sufficient disk space
3. Reduce resolution for faster preview

### Performance Issues

- Use lower FPS during development
- Reduce frame count for testing
- Optimize image assets

### Memory Issues

```bash
NODE_OPTIONS=--max-old-space-size=4096 npm run build
```

## Resources

- [Remotion Documentation](https://www.remotion.dev/docs/)
- [Remotion Examples](https://www.remotion.dev/examples)
- [Remotion GitHub](https://github.com/remotion-dev/remotion)

## License

MIT
