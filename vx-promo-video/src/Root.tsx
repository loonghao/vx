import React from 'react';
import {Composition, registerRoot} from 'remotion';
import {IntroScene} from './scenes/IntroScene';
import {ProblemScene} from './scenes/ProblemScene';
import {SolutionScene} from './scenes/SolutionScene';
import {MCPScene} from './scenes/MCPScene';
import {FeaturesScene} from './scenes/FeaturesScene';
import {ProvidersScene} from './scenes/ProvidersScene';
import {CallToActionScene} from './scenes/CallToActionScene';
import {FullVideo} from './FullVideo';

// Scene durations in frames (30 fps)
const INTRO_DURATION = 120;        // 4 seconds
const PROBLEM_DURATION = 300;      // 10 seconds
const SOLUTION_DURATION = 300;     // 10 seconds
const MCP_DURATION = 210;          // 7 seconds (faster paced)
const PROVIDERS_DURATION = 270;    // 9 seconds
const FEATURES_DURATION = 210;     // 7 seconds (faster paced)
const CTA_DURATION = 210;          // 7 seconds

// Total duration
const TOTAL_DURATION = INTRO_DURATION + PROBLEM_DURATION + SOLUTION_DURATION + MCP_DURATION + PROVIDERS_DURATION + FEATURES_DURATION + CTA_DURATION;

export const RemotionRoot: React.FC = () => {
  return (
    <>
      {/* Individual scenes for preview/editing */}
      <Composition
        id="vx-intro"
        component={IntroScene}
        durationInFrames={INTRO_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-problem"
        component={ProblemScene}
        durationInFrames={PROBLEM_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-solution"
        component={SolutionScene}
        durationInFrames={SOLUTION_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-mcp"
        component={MCPScene}
        durationInFrames={MCP_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-providers"
        component={ProvidersScene}
        durationInFrames={PROVIDERS_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-features"
        component={FeaturesScene}
        durationInFrames={FEATURES_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      <Composition
        id="vx-cta"
        component={CallToActionScene}
        durationInFrames={CTA_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />

      {/* Full video - all scenes combined with transitions */}
      <Composition
        id="vx-full"
        component={FullVideo}
        durationInFrames={TOTAL_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};

registerRoot(RemotionRoot);
