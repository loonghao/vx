import React from 'react';
import {AbsoluteFill, useCurrentFrame, interpolate, Easing, spring} from 'remotion';

interface GlowRingProps {
  delay?: number;
  color?: string;
  size?: number;
  thickness?: number;
  duration?: number;
}

export const GlowRing: React.FC<GlowRingProps> = ({
  delay = 0,
  color = '#67e8f9',
  size = 400,
  thickness = 2,
  duration = 60,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const scale = interpolate(effectiveFrame, [0, duration], [0.2, 2.5], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  const opacity = interpolate(effectiveFrame, [0, duration * 0.3, duration], [0, 0.8, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  if (effectiveFrame < 0) return null;

  return (
    <div
      style={{
        position: 'absolute',
        left: '50%',
        top: '50%',
        width: size,
        height: size,
        transform: `translate(-50%, -50%) scale(${scale})`,
        borderRadius: '50%',
        border: `${thickness}px solid ${color}`,
        opacity,
        boxShadow: `
          0 0 20px ${color},
          0 0 40px ${color},
          inset 0 0 20px ${color}
        `,
        pointerEvents: 'none',
      }}
    />
  );
};

interface ShockwaveProps {
  trigger?: number;
  count?: number;
  interval?: number;
}

export const Shockwave: React.FC<ShockwaveProps> = ({
  trigger = 0,
  count = 3,
  interval = 10,
}) => {
  const rings = [];
  for (let i = 0; i < count; i++) {
    rings.push(
      <GlowRing
        key={i}
        delay={trigger + i * interval}
        color={i % 2 === 0 ? '#67e8f9' : '#a78bfa'}
        size={300 + i * 50}
        thickness={3 - i * 0.5}
        duration={60 + i * 10}
      />
    );
  }

  return <AbsoluteFill>{rings}</AbsoluteFill>;
};

interface PulseGlowProps {
  delay?: number;
  color?: string;
}

export const PulseGlow: React.FC<PulseGlowProps> = ({
  delay = 0,
  color = '#67e8f9',
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const pulse = spring({
    frame: effectiveFrame,
    fps: 30,
    config: {
      damping: 10,
      stiffness: 50,
      mass: 0.5,
    },
  });

  const glow = interpolate(pulse, [0, 1], [0, 1]);
  const breathe = Math.sin(effectiveFrame * 0.05) * 0.2 + 0.8;

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
      }}
    >
      <div
        style={{
          width: 200,
          height: 200,
          borderRadius: '50%',
          background: `radial-gradient(circle, ${color}40 0%, transparent 70%)`,
          transform: `scale(${glow * breathe * 3})`,
          opacity: glow * 0.6,
          filter: 'blur(30px)',
        }}
      />
    </AbsoluteFill>
  );
};

interface FloatingOrbProps {
  x: number;
  y: number;
  size?: number;
  color?: string;
  delay?: number;
  speed?: number;
}

export const FloatingOrb: React.FC<FloatingOrbProps> = ({
  x,
  y,
  size = 100,
  color = '#67e8f9',
  delay = 0,
  speed = 1,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const float = Math.sin(effectiveFrame * 0.02 * speed) * 20;
  const float2 = Math.cos(effectiveFrame * 0.015 * speed) * 15;

  const opacity = interpolate(effectiveFrame, [0, 30], [0, 0.6], {
    extrapolateRight: 'clamp',
  });

  const breathe = Math.sin(effectiveFrame * 0.03) * 0.15 + 1;

  return (
    <div
      style={{
        position: 'absolute',
        left: `${x}%`,
        top: `${y}%`,
        transform: `translate(-50%, -50%) translate(${float}px, ${float2}px) scale(${breathe})`,
        width: size,
        height: size,
        borderRadius: '50%',
        background: `radial-gradient(circle at 30% 30%, ${color}60, ${color}20 50%, transparent 70%)`,
        boxShadow: `
          0 0 ${size * 0.5}px ${color}40,
          inset 0 0 ${size * 0.3}px ${color}30
        `,
        opacity,
        filter: `blur(${size * 0.05}px)`,
        pointerEvents: 'none',
      }}
    />
  );
};
