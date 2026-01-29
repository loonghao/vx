import React from 'react';
import {
  useCurrentFrame,
  interpolate,
  Easing,
  AbsoluteFill,
} from 'remotion';

/**
 * Cinematic scene transition effects
 */

interface TransitionOverlayProps {
  type: 'fade' | 'wipe' | 'zoom' | 'blur' | 'flash';
  duration?: number;
  direction?: 'in' | 'out';
}

export const TransitionOverlay: React.FC<TransitionOverlayProps> = ({
  type,
  duration = 20,
  direction = 'in',
}) => {
  const frame = useCurrentFrame();
  
  const progress = direction === 'in'
    ? interpolate(frame, [0, duration], [0, 1], { extrapolateRight: 'clamp' })
    : interpolate(frame, [0, duration], [1, 0], { extrapolateRight: 'clamp' });
  
  switch (type) {
    case 'fade':
      return (
        <AbsoluteFill
          style={{
            backgroundColor: '#000',
            opacity: 1 - progress,
            pointerEvents: 'none',
          }}
        />
      );
      
    case 'wipe':
      return (
        <AbsoluteFill
          style={{
            background: `linear-gradient(90deg, transparent ${progress * 100}%, #000 ${progress * 100}%)`,
            pointerEvents: 'none',
          }}
        />
      );
      
    case 'zoom':
      const scale = interpolate(progress, [0, 1], [1.5, 1], {
        easing: Easing.out(Easing.cubic),
      });
      return (
        <AbsoluteFill
          style={{
            backgroundColor: '#000',
            opacity: 1 - progress,
            transform: `scale(${scale})`,
            pointerEvents: 'none',
          }}
        />
      );
      
    case 'blur':
      const blur = interpolate(progress, [0, 1], [20, 0]);
      return (
        <AbsoluteFill
          style={{
            backdropFilter: `blur(${blur}px)`,
            WebkitBackdropFilter: `blur(${blur}px)`,
            opacity: 1 - progress * 0.5,
            pointerEvents: 'none',
          }}
        />
      );
      
    case 'flash':
      const flashOpacity = interpolate(progress, [0, 0.3, 1], [0, 1, 0]);
      return (
        <AbsoluteFill
          style={{
            backgroundColor: '#fff',
            opacity: flashOpacity * 0.3,
            pointerEvents: 'none',
          }}
        />
      );
      
    default:
      return null;
  }
};

/**
 * Cinematic letterbox effect (black bars)
 */
interface LetterboxProps {
  intensity?: number;
}

export const Letterbox: React.FC<LetterboxProps> = ({ intensity = 0.08 }) => {
  return (
    <>
      <AbsoluteFill
        style={{
          background: `linear-gradient(180deg, #000 0%, transparent ${intensity * 100}%)`,
          pointerEvents: 'none',
        }}
      />
      <AbsoluteFill
        style={{
          background: `linear-gradient(0deg, #000 0%, transparent ${intensity * 100}%)`,
          pointerEvents: 'none',
        }}
      />
    </>
  );
};

/**
 * Light leak / lens flare effect
 */
interface LightLeakProps {
  delay?: number;
  duration?: number;
  color?: string;
  position?: 'left' | 'right' | 'center';
}

export const LightLeak: React.FC<LightLeakProps> = ({
  delay = 0,
  duration = 30,
  color = '#67e8f9',
  position = 'right',
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const progress = interpolate(effectiveFrame, [0, duration / 2, duration], [0, 1, 0], {
    extrapolateRight: 'clamp',
  });
  
  const positionStyle = {
    left: { left: '-20%', transform: 'rotate(-30deg)' },
    right: { right: '-20%', transform: 'rotate(30deg)' },
    center: { left: '30%', transform: 'rotate(15deg)' },
  };
  
  if (progress <= 0) return null;
  
  return (
    <AbsoluteFill
      style={{
        overflow: 'hidden',
        pointerEvents: 'none',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: '-50%',
          width: '200%',
          height: '200%',
          background: `linear-gradient(135deg, transparent 40%, ${color}40 50%, transparent 60%)`,
          opacity: progress * 0.5,
          filter: 'blur(60px)',
          ...positionStyle[position],
        }}
      />
    </AbsoluteFill>
  );
};

/**
 * Particle burst effect for dramatic moments
 */
interface ParticleBurstProps {
  delay?: number;
  count?: number;
  color?: string;
}

export const ParticleBurst: React.FC<ParticleBurstProps> = ({
  delay = 0,
  count = 20,
  color = '#67e8f9',
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  if (effectiveFrame <= 0) return null;
  
  const particles = Array.from({ length: count }, (_, i) => {
    const angle = (i / count) * Math.PI * 2;
    const distance = interpolate(effectiveFrame, [0, 60], [0, 300], {
      extrapolateRight: 'clamp',
      easing: Easing.out(Easing.cubic),
    });
    const opacity = interpolate(effectiveFrame, [0, 20, 60], [0, 1, 0], {
      extrapolateRight: 'clamp',
    });
    const scale = interpolate(effectiveFrame, [0, 60], [1, 0.2], {
      extrapolateRight: 'clamp',
    });
    
    const x = Math.cos(angle) * distance;
    const y = Math.sin(angle) * distance;
    
    return (
      <div
        key={i}
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          width: 8,
          height: 8,
          borderRadius: '50%',
          backgroundColor: color,
          opacity,
          transform: `translate(-50%, -50%) translate(${x}px, ${y}px) scale(${scale})`,
          boxShadow: `0 0 10px ${color}`,
        }}
      />
    );
  });
  
  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      {particles}
    </AbsoluteFill>
  );
};

/**
 * Radial pulse effect
 */
interface RadialPulseProps {
  delay?: number;
  color?: string;
  maxSize?: number;
}

export const RadialPulse: React.FC<RadialPulseProps> = ({
  delay = 0,
  color = '#67e8f9',
  maxSize = 800,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const size = interpolate(effectiveFrame, [0, 60], [0, maxSize], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });
  
  const opacity = interpolate(effectiveFrame, [0, 10, 60], [0, 0.5, 0], {
    extrapolateRight: 'clamp',
  });
  
  if (opacity <= 0) return null;
  
  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        pointerEvents: 'none',
      }}
    >
      <div
        style={{
          width: size,
          height: size,
          borderRadius: '50%',
          border: `2px solid ${color}`,
          opacity,
          boxShadow: `0 0 40px ${color}40, inset 0 0 40px ${color}20`,
        }}
      />
    </AbsoluteFill>
  );
};

/**
 * Cinematic bars that animate in/out
 */
interface CinematicBarsProps {
  show: boolean;
  barHeight?: number;
}

export const CinematicBars: React.FC<CinematicBarsProps> = ({
  show,
  barHeight = 100,
}) => {
  const frame = useCurrentFrame();
  
  const height = show
    ? interpolate(frame, [0, 30], [0, barHeight], {
        extrapolateRight: 'clamp',
        easing: Easing.out(Easing.cubic),
      })
    : interpolate(frame, [0, 30], [barHeight, 0], {
        extrapolateRight: 'clamp',
        easing: Easing.in(Easing.cubic),
      });
  
  return (
    <>
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height,
          backgroundColor: '#000',
          zIndex: 1000,
        }}
      />
      <div
        style={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          right: 0,
          height,
          backgroundColor: '#000',
          zIndex: 1000,
        }}
      />
    </>
  );
};
