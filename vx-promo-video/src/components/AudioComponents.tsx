import React from 'react';
import {Audio, interpolate, useCurrentFrame, staticFile} from 'remotion';

interface BackgroundMusicProps {
  src: string;
  volume?: number;
  fadeInDuration?: number;
  fadeOutDuration?: number;
  startFrom?: number;
}

/**
 * Background Music component with fade in/out support
 * 
 * To use this, you need to add a music file to public/audio/
 * 
 * Recommended free epic/cinematic music sources:
 * - https://pixabay.com/music/search/epic%20cinematic/
 * - https://www.bensound.com/royalty-free-music
 * - https://mixkit.co/free-stock-music/
 * 
 * Download a suitable track and place it in public/audio/bgm.mp3
 */
export const BackgroundMusic: React.FC<BackgroundMusicProps> = ({
  src,
  volume = 0.3,
  fadeInDuration = 60,
  fadeOutDuration = 90,
  startFrom = 0,
}) => {
  const frame = useCurrentFrame();
  
  // Calculate fade in
  const fadeIn = interpolate(frame, [0, fadeInDuration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  
  // Note: Fade out would need to know the total duration
  // which is handled at the composition level
  
  const currentVolume = volume * fadeIn;
  
  return (
    <Audio
      src={staticFile(src)}
      volume={currentVolume}
      startFrom={startFrom}
    />
  );
};

interface SoundEffectProps {
  src: string;
  volume?: number;
  delay?: number;
}

/**
 * Sound Effect component for short audio clips
 */
export const SoundEffect: React.FC<SoundEffectProps> = ({
  src,
  volume = 0.5,
  delay = 0,
}) => {
  const frame = useCurrentFrame();
  
  // Only play after delay
  if (frame < delay) {
    return null;
  }
  
  return (
    <Audio
      src={staticFile(src)}
      volume={volume}
      startFrom={0}
    />
  );
};

// Sound effect for typing
export const TypingSound: React.FC<{delay?: number}> = ({delay = 0}) => {
  return (
    <SoundEffect
      src="audio/typing.mp3"
      volume={0.2}
      delay={delay}
    />
  );
};

// Sound effect for success/completion
export const SuccessSound: React.FC<{delay?: number}> = ({delay = 0}) => {
  return (
    <SoundEffect
      src="audio/success.mp3"
      volume={0.3}
      delay={delay}
    />
  );
};

// Whoosh transition sound
export const WhooshSound: React.FC<{delay?: number}> = ({delay = 0}) => {
  return (
    <SoundEffect
      src="audio/whoosh.mp3"
      volume={0.2}
      delay={delay}
    />
  );
};
