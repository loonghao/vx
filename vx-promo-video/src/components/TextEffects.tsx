import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
  Easing,
} from 'remotion';
import {typography} from '../fonts';

interface BigRevealProps {
  text: string;
  delay?: number;
  color?: string;
  fontSize?: number;
}

export const BigReveal: React.FC<BigRevealProps> = ({
  text,
  delay = 0,
  color = '#ffffff',
  fontSize = 120,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  // Epic reveal animation
  const scale = spring({
    frame: effectiveFrame,
    fps: 30,
    config: {
      damping: 15,
      stiffness: 80,
      mass: 0.8,
    },
  });

  const y = interpolate(scale, [0, 1], [100, 0]);
  const opacity = interpolate(scale, [0, 0.3, 1], [0, 1, 1]);

  // Glow pulse
  const glowPulse = Math.sin(effectiveFrame * 0.1) * 0.3 + 0.7;
  const glowIntensity = interpolate(effectiveFrame, [0, 30], [0, 1], {
    extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
      }}
    >
      {/* Glow behind text */}
      <div
        style={{
          position: 'absolute',
          width: '80%',
          height: 200,
          background: `radial-gradient(ellipse, ${color}30 0%, transparent 70%)`,
          filter: 'blur(60px)',
          opacity: glowIntensity * glowPulse,
        }}
      />

      <h1
        style={{
          ...typography.title,
          fontSize,
          color,
          textAlign: 'center',
          transform: `translateY(${y}px) scale(${scale})`,
          opacity,
          textShadow: `
            0 0 ${glowIntensity * 40}px ${color}80,
            0 0 ${glowIntensity * 80}px ${color}40
          `,
          letterSpacing: '-0.03em',
        }}
      >
        {text}
      </h1>
    </AbsoluteFill>
  );
};

interface CountdownRevealProps {
  numbers: string[];
  delay?: number;
  interval?: number;
}

export const CountdownReveal: React.FC<CountdownRevealProps> = ({
  numbers,
  delay = 0,
  interval = 20,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const currentIndex = Math.min(
    Math.floor(effectiveFrame / interval),
    numbers.length - 1
  );

  return (
    <AbsoluteFill>
      {numbers.map((num, index) => {
        const showStart = index * interval;
        const showEnd = (index + 1) * interval;
        const isCurrentOrPast = effectiveFrame >= showStart;
        const isCurrent = index === currentIndex;

        if (!isCurrentOrPast) return null;

        const localFrame = effectiveFrame - showStart;
        const scale = spring({
          frame: localFrame,
          fps: 30,
          config: {damping: 12, stiffness: 100, mass: 0.5},
        });

        const opacity = isCurrent
          ? interpolate(localFrame, [0, 5], [0, 1], {extrapolateRight: 'clamp'})
          : interpolate(
              effectiveFrame,
              [showEnd - 5, showEnd],
              [1, 0],
              {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'}
            );

        return (
          <AbsoluteFill
            key={index}
            style={{
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              opacity,
            }}
          >
            <span
              style={{
                ...typography.title,
                fontSize: 200,
                color: '#67e8f9',
                transform: `scale(${scale})`,
                textShadow: '0 0 60px rgba(103, 232, 249, 0.5)',
              }}
            >
              {num}
            </span>
          </AbsoluteFill>
        );
      })}
    </AbsoluteFill>
  );
};

interface TextFlashProps {
  text: string;
  trigger: number;
  duration?: number;
}

export const TextFlash: React.FC<TextFlashProps> = ({
  text,
  trigger,
  duration = 30,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = frame - trigger;

  if (effectiveFrame < 0 || effectiveFrame > duration) return null;

  const scale = interpolate(effectiveFrame, [0, 5, duration], [0.8, 1.1, 1], {
    easing: Easing.out(Easing.cubic),
  });

  const opacity = interpolate(effectiveFrame, [0, 5, duration - 5, duration], [0, 1, 1, 0.8]);

  const flashOpacity = interpolate(effectiveFrame, [0, 3, 8], [0.3, 0, 0], {
    extrapolateRight: 'clamp',
  });

  return (
    <>
      {/* Flash effect */}
      <AbsoluteFill
        style={{
          backgroundColor: '#67e8f9',
          opacity: flashOpacity,
        }}
      />
      
      <AbsoluteFill
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          opacity,
        }}
      >
        <h1
          style={{
            ...typography.title,
            fontSize: 100,
            background: 'linear-gradient(135deg, #ffffff 0%, #67e8f9 50%, #a78bfa 100%)',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            backgroundClip: 'text',
            transform: `scale(${scale})`,
            textShadow: 'none',
            filter: 'drop-shadow(0 0 30px rgba(103, 232, 249, 0.5))',
          }}
        >
          {text}
        </h1>
      </AbsoluteFill>
    </>
  );
};
