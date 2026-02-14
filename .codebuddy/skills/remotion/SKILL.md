---
name: remotion
description: |
  Create videos programmatically using React. This skill helps with Remotion setup,
  composition creation, rendering, and best practices for building video applications.
---

# Remotion Skill

Create videos programmatically using React. This skill provides guidance and tools for working with Remotion, a framework that allows you to build video content with React components.

## Overview

Remotion is a React-based video creation framework that lets you:
- Build videos with familiar React concepts
- Use JavaScript/TypeScript for dynamic content
- Render videos programmatically
- Integrate video creation into applications
- Build parameterizable and reusable video components

## When to Use This Skill

Use the Remotion skill when:
- Creating a new Remotion project
- Adding video generation to an existing React project
- Setting up Remotion compositions
- Configuring video rendering pipelines
- Creating reusable video components
- Integrating video rendering with server-side applications

## Quick Start

### Creating a New Remotion Project

```bash
# Using npx (recommended)
npx create-video my-video

# Or using npm init
npm init video my-video

# Or using yarn
yarn create video my-video

# Or using pnpm
pnpm create video my-video
```

### Adding Remotion to Existing Project

```bash
# Install core package
npm install remotion

# Install CLI (for development)
npm install -D @remotion/cli

# Optional: Install renderer for server-side rendering
npm install @remotion/renderer
```

## Project Structure

A typical Remotion project structure:

```
my-video/
├── src/
│   ├── Root.tsx           # Root composition
│   ├── Video.tsx          # Main video component
│   ├── components/        # Reusable components
│   │   ├── Background.tsx
│   │   ├── Text.tsx
│   │   └── ...
│   ├── compositions/      # Video compositions
│   │   ├── MainComp.tsx
│   │   └── ...
│   └── index.tsx         # Entry point
├── package.json
├── remotion.config.ts    # Configuration
└── public/
    └── assets/
```

## Core Concepts

### 1. Composition

A composition is a video scene with specific duration and frame rate:

```tsx
import {Composition} from 'remotion';
import {MyVideo} from './Video';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="my-video"
        component={MyVideo}
        durationInFrames={300}  // 10 seconds at 30fps
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};
```

### 2. Video Component

```tsx
import {AbsoluteFill, useCurrentFrame, useVideoConfig} from 'remotion';

export const MyVideo: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();

  // Animation based on frame
  const opacity = frame / 60;  // Fade in over 2 seconds

  return (
    <AbsoluteFill style={{backgroundColor: 'white'}}>
      <AbsoluteFill style={{opacity}}>
        <h1>Hello, Remotion!</h1>
        <p>Frame: {frame}</p>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
```

### 3. Configuration

Create `remotion.config.ts`:

```ts
import {Config} from '@remotion/cli/config';

Config.setVideoImageFormat('jpeg');
Config.setOverwriteOutput(true);
Config.setBrowserExecutable('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome');
```

## Common Patterns

### Animations

Using `spring` for smooth animations:

```tsx
import {spring, useCurrentFrame, useVideoConfig} from 'remotion';

export const AnimatedElement: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();

  const scale = spring({
    frame,
    fps,
    config: {
      damping: 200,
      stiffness: 100,
    },
  });

  return (
    <div style={{transform: `scale(${scale})`}}>
      Animated Content
    </div>
  );
};
```

### Input Props

Pass data to compositions:

```tsx
// Composition definition
<Composition
  id="my-video"
  component={MyVideo}
  durationInFrames={300}
  fps={30}
  width={1920}
  height={1080}
  defaultProps={{
    title: "Hello World",
    color: "blue",
    speed: 1.0,
  }}
/>

// Usage
export const MyVideo: React.FC<{title: string; color: string; speed: number}> = ({
  title,
  color,
  speed,
}) => {
  return <div style={{color}}>{title}</div>;
};
```

### Sequence

Chain animations sequentially:

```tsx
import {Sequence} from 'remotion';

export const SequenceExample: React.FC = () => {
  return (
    <AbsoluteFill>
      <Sequence from={0} durationInFrames={60}>
        <FirstScene />
      </Sequence>
      <Sequence from={60} durationInFrames={60}>
        <SecondScene />
      </Sequence>
      <Sequence from={120} durationInFrames={180}>
        <FinalScene />
      </Sequence>
    </AbsoluteFill>
  );
};
```

### Loop

Repeat animations:

```tsx
import {Loop} from 'remotion';

export const LoopExample: React.FC = () => {
  return (
    <Loop durationInFrames={30} times={10}>
      <BouncingBall />
    </Loop>
  );
};
```

## Rendering Videos

### Preview in Browser

```bash
npx remotion preview src/Root.tsx
```

### Render to MP4

```bash
npx remotion render src/Root.tsx my-video --output=output.mp4
```

### Render with Parameters

```bash
npx remotion render src/Root.tsx my-video --output=output.mp4 --props='{"title":"Custom Title"}'
```

### Server-Side Rendering

```ts
import {bundle} from '@remotion/bundler';
import {renderMedia, selectComposition} from '@remotion/renderer';

async function renderVideo() {
  // Bundle the video
  const bundleLocation = await bundle({
    entryPoint: './src/Root.tsx',
    webpackOverride: (config) => config,
  });

  // Select composition
  const composition = await selectComposition({
    serveUrl: bundleLocation,
    id: 'my-video',
    inputProps: {},
  });

  // Render
  await renderMedia({
    composition,
    serveUrl: bundleLocation,
    codec: 'h264',
    outputLocation: `output/${composition.id}.mp4`,
    inputProps: {},
  });
}
```

## Best Practices

### 1. Component Structure

- Keep components small and reusable
- Use TypeScript for type safety
- Extract animations to separate hooks or utilities

### 2. Performance

- Use `staticDuration` when composition length doesn't change
- Optimize images and assets
- Use `useCurrentFrame` sparingly in complex components

### 3. Organization

```
src/
├── compositions/      # Define video scenes
├── components/        # Reusable UI elements
├── hooks/           # Custom hooks for animations
├── utils/           # Helper functions
└── assets/          # Images, fonts, etc.
```

### 4. Asset Management

- Use `public/` folder for static assets
- Import images directly in components
- Optimize images before including in video

## Troubleshooting

### Video Not Rendering

1. Check Chrome/Chromium is installed
2. Verify composition ID matches
3. Ensure output directory exists
4. Check for console errors in preview

### Performance Issues

1. Reduce composition size during development
2. Use lower resolution for testing
3. Minimize expensive computations in render
4. Use `staticDuration` when possible

### Memory Issues

```bash
# Reduce memory usage
NODE_OPTIONS=--max-old-space-size=4096 npx remotion render src/Root.tsx my-video
```

## Advanced Features

### Audio Integration

```tsx
import {Audio, useAudioData} from 'remotion';

export const AudioExample: React.FC = () => {
  const {audioData} = useAudioData('/path/to/audio.mp3');

  return (
    <>
      <Audio src="/path/to/audio.mp3" />
      {/* Visualize audio if needed */}
    </>
  );
};
```

### Transitions

```tsx
import {TransitionSeries} from '@remotion/transitions';

export const TransitionExample: React.FC = () => {
  return (
    <TransitionSeries>
      <TransitionSeries.Transition
        in={<FirstScene />}
        out={<SecondScene />}
      >
        <TransitionSeries.Sequence name="first" durationInFrames={60}>
          <FirstScene />
        </TransitionSeries.Sequence>
        <TransitionSeries.Sequence name="second" durationInFrames={60}>
          <SecondScene />
        </TransitionSeries.Sequence>
      </TransitionSeries.Transition>
    </TransitionSeries>
  );
};
```

### Lambda Rendering

For serverless video rendering:

```bash
npm install @remotion/lambda

# Deploy to Lambda
npx remotion lambda sites create src/ --site-name=my-site

# Render on Lambda
npx remotion lambda render my-video --output=output.mp4
```

## Resources

- [Remotion Documentation](https://www.remotion.dev/docs/)
- [Remotion Examples](https://www.remotion.dev/examples)
- [Remotion GitHub](https://github.com/remotion-dev/remotion)
- [Remotion Discord](https://discord.com/invite/VX6sQJ9)
- [Remotion Twitter](https://twitter.com/remotion_dev)

## Getting Help

When working with Remotion:
1. Check the official documentation
2. Review the examples repository
3. Join the Discord community
4. Search GitHub issues
5. Ask in the Remotion community forums

## Next Steps

After setting up your Remotion project:
1. Create your first composition
2. Build reusable components
3. Test animations in the preview
4. Render your first video
5. Optimize for performance
6. Deploy your rendering pipeline
