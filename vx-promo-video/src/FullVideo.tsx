import React from 'react';
import {AbsoluteFill, Sequence, useCurrentFrame, interpolate, Easing, Audio, staticFile} from 'remotion';
import {IntroScene} from './scenes/IntroScene';
import {ProblemScene} from './scenes/ProblemScene';
import {SolutionScene} from './scenes/SolutionScene';
import {MCPScene} from './scenes/MCPScene';
import {ProvidersScene} from './scenes/ProvidersScene';
import {FeaturesScene} from './scenes/FeaturesScene';
import {CallToActionScene} from './scenes/CallToActionScene';

// Scene durations in frames (30 fps)
const INTRO_DURATION = 120;        // 4 seconds
const PROBLEM_DURATION = 300;      // 10 seconds
const SOLUTION_DURATION = 300;     // 10 seconds
const MCP_DURATION = 210;          // 7 seconds (faster paced)
const PROVIDERS_DURATION = 270;    // 9 seconds
const FEATURES_DURATION = 210;     // 7 seconds (faster paced)
const CTA_DURATION = 210;          // 7 seconds

// Transition duration
const TRANSITION_FRAMES = 20;      // ~0.67 seconds for smoother transitions

// Total duration
const TOTAL_DURATION = INTRO_DURATION + PROBLEM_DURATION + SOLUTION_DURATION + 
                       MCP_DURATION + PROVIDERS_DURATION + FEATURES_DURATION + CTA_DURATION;

// Background music component with fade in/out
const BackgroundMusic: React.FC = () => {
  const frame = useCurrentFrame();
  
  // Fade in over first 2 seconds
  const fadeIn = interpolate(frame, [0, 60], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  
  // Fade out over last 3 seconds
  const fadeOut = interpolate(frame, [TOTAL_DURATION - 90, TOTAL_DURATION], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  
  const volume = Math.min(fadeIn, fadeOut) * 0.25; // Max 25% volume
  
  return (
    <Audio
      src={staticFile('audio/epic-cinematic.mp3')}
      volume={volume}
      startFrom={0}
    />
  );
};

// Scene wrapper with fade transition
interface SceneWrapperProps {
  children: React.ReactNode;
  durationInFrames: number;
}

const SceneWrapper: React.FC<SceneWrapperProps> = ({children, durationInFrames}) => {
  const frame = useCurrentFrame();
  
  // Fade in at start
  const fadeIn = interpolate(frame, [0, TRANSITION_FRAMES], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });
  
  // Fade out at end
  const fadeOut = interpolate(
    frame,
    [durationInFrames - TRANSITION_FRAMES, durationInFrames],
    [1, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.in(Easing.cubic),
    }
  );
  
  const opacity = Math.min(fadeIn, fadeOut);
  
  // Subtle scale for cinematic feel
  const scale = interpolate(frame, [0, TRANSITION_FRAMES], [1.02, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });
  
  return (
    <AbsoluteFill style={{
      opacity,
      transform: `scale(${scale})`,
    }}>
      {children}
    </AbsoluteFill>
  );
};

export const FullVideo: React.FC = () => {
  // Calculate start frames for each scene
  const problemStart = INTRO_DURATION;
  const solutionStart = problemStart + PROBLEM_DURATION;
  const mcpStart = solutionStart + SOLUTION_DURATION;
  const providersStart = mcpStart + MCP_DURATION;
  const featuresStart = providersStart + PROVIDERS_DURATION;
  const ctaStart = featuresStart + FEATURES_DURATION;

  return (
    <AbsoluteFill style={{backgroundColor: '#000000'}}>
      {/* Background Music */}
      <BackgroundMusic />
      
      {/* Scene 1: Introduction */}
      <Sequence from={0} durationInFrames={INTRO_DURATION + TRANSITION_FRAMES}>
        <SceneWrapper durationInFrames={INTRO_DURATION + TRANSITION_FRAMES}>
          <IntroScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 2: Problem (Before VX) */}
      <Sequence from={problemStart - TRANSITION_FRAMES} durationInFrames={PROBLEM_DURATION + TRANSITION_FRAMES * 2}>
        <SceneWrapper durationInFrames={PROBLEM_DURATION + TRANSITION_FRAMES * 2}>
          <ProblemScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 3: Solution (With VX) */}
      <Sequence from={solutionStart - TRANSITION_FRAMES} durationInFrames={SOLUTION_DURATION + TRANSITION_FRAMES * 2}>
        <SceneWrapper durationInFrames={SOLUTION_DURATION + TRANSITION_FRAMES * 2}>
          <SolutionScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 4: MCP Configuration */}
      <Sequence from={mcpStart - TRANSITION_FRAMES} durationInFrames={MCP_DURATION + TRANSITION_FRAMES * 2}>
        <SceneWrapper durationInFrames={MCP_DURATION + TRANSITION_FRAMES * 2}>
          <MCPScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 5: Providers Ecosystem */}
      <Sequence from={providersStart - TRANSITION_FRAMES} durationInFrames={PROVIDERS_DURATION + TRANSITION_FRAMES * 2}>
        <SceneWrapper durationInFrames={PROVIDERS_DURATION + TRANSITION_FRAMES * 2}>
          <ProvidersScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 6: Features */}
      <Sequence from={featuresStart - TRANSITION_FRAMES} durationInFrames={FEATURES_DURATION + TRANSITION_FRAMES * 2}>
        <SceneWrapper durationInFrames={FEATURES_DURATION + TRANSITION_FRAMES * 2}>
          <FeaturesScene />
        </SceneWrapper>
      </Sequence>

      {/* Scene 7: Call to Action */}
      <Sequence from={ctaStart - TRANSITION_FRAMES} durationInFrames={CTA_DURATION + TRANSITION_FRAMES}>
        <SceneWrapper durationInFrames={CTA_DURATION + TRANSITION_FRAMES}>
          <CallToActionScene />
        </SceneWrapper>
      </Sequence>
      
      {/* Film grain overlay for cinematic effect */}
      <AbsoluteFill
        style={{
          background: 'url(data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIzMDAiIGhlaWdodD0iMzAwIj48ZmlsdGVyIGlkPSJub2lzZSI+PGZlVHVyYnVsZW5jZSB0eXBlPSJmcmFjdGFsTm9pc2UiIGJhc2VGcmVxdWVuY3k9IjAuNiIgbnVtT2N0YXZlcz0iMyIvPjwvZmlsdGVyPjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbHRlcj0idXJsKCNub2lzZSkiLz48L3N2Zz4=)',
          opacity: 0.03,
          mixBlendMode: 'overlay',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
