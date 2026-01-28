import React from 'react';
import {
  useCurrentFrame,
  interpolate,
  spring,
  Easing,
} from 'remotion';

// Smooth easing functions
export const smoothEasings = {
  // Apple-style smooth deceleration
  appleOut: (t: number) => 1 - Math.pow(1 - t, 4),
  // Elastic bounce
  elastic: (t: number) => {
    const c4 = (2 * Math.PI) / 3;
    return t === 0 ? 0 : t === 1 ? 1 :
      Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
  },
  // Smooth overshoot
  backOut: (t: number) => {
    const c1 = 1.70158;
    const c3 = c1 + 1;
    return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
  },
};

// Staggered animation hook
export function useStaggeredAnimation(
  itemCount: number,
  startFrame: number,
  staggerDelay: number = 8,
  config = { damping: 15, stiffness: 80, mass: 0.5 }
) {
  const frame = useCurrentFrame();
  
  return Array.from({ length: itemCount }, (_, index) => {
    const delay = startFrame + index * staggerDelay;
    const effectiveFrame = Math.max(0, frame - delay);
    
    const progress = spring({
      frame: effectiveFrame,
      fps: 30,
      config,
    });
    
    return {
      opacity: interpolate(progress, [0, 1], [0, 1]),
      translateY: interpolate(progress, [0, 1], [30, 0]),
      scale: interpolate(progress, [0, 1], [0.95, 1]),
      progress,
      isVisible: effectiveFrame > 0,
    };
  });
}

// Smooth fade transition
interface FadeInProps {
  delay?: number;
  duration?: number;
  children: React.ReactNode;
  direction?: 'up' | 'down' | 'left' | 'right' | 'none';
  distance?: number;
}

export const FadeIn: React.FC<FadeInProps> = ({
  delay = 0,
  duration = 30,
  children,
  direction = 'up',
  distance = 30,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const progress = interpolate(effectiveFrame, [0, duration], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });
  
  const getTransform = () => {
    const translate = interpolate(progress, [0, 1], [distance, 0]);
    switch (direction) {
      case 'up': return `translateY(${translate}px)`;
      case 'down': return `translateY(-${translate}px)`;
      case 'left': return `translateX(${translate}px)`;
      case 'right': return `translateX(-${translate}px)`;
      default: return 'none';
    }
  };
  
  return (
    <div
      style={{
        opacity: progress,
        transform: getTransform(),
      }}
    >
      {children}
    </div>
  );
};

// Smooth scale reveal
interface ScaleRevealProps {
  delay?: number;
  children: React.ReactNode;
  origin?: string;
}

export const ScaleReveal: React.FC<ScaleRevealProps> = ({
  delay = 0,
  children,
  origin = 'center',
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const scale = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 12, stiffness: 100, mass: 0.5 },
  });
  
  const opacity = interpolate(effectiveFrame, [0, 15], [0, 1], {
    extrapolateRight: 'clamp',
  });
  
  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale})`,
        transformOrigin: origin,
      }}
    >
      {children}
    </div>
  );
};

// Icon badge component with animation
interface AnimatedBadgeProps {
  icon: string;
  label: string;
  color: string;
  delay?: number;
  size?: 'small' | 'medium' | 'large';
}

export const AnimatedBadge: React.FC<AnimatedBadgeProps> = ({
  icon,
  label,
  color,
  delay = 0,
  size = 'medium',
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const scale = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 12, stiffness: 120, mass: 0.4 },
  });
  
  const opacity = interpolate(effectiveFrame, [0, 10], [0, 1], {
    extrapolateRight: 'clamp',
  });
  
  const glowIntensity = interpolate(effectiveFrame, [0, 30], [0, 1], {
    extrapolateRight: 'clamp',
  });
  
  const sizes = {
    small: { padding: '6px 12px', fontSize: 12, iconSize: 14, gap: 6 },
    medium: { padding: '8px 16px', fontSize: 14, iconSize: 18, gap: 8 },
    large: { padding: '12px 20px', fontSize: 16, iconSize: 22, gap: 10 },
  };
  
  const s = sizes[size];
  
  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale})`,
        display: 'inline-flex',
        alignItems: 'center',
        gap: s.gap,
        padding: s.padding,
        borderRadius: 100,
        backgroundColor: `${color}15`,
        border: `1px solid ${color}30`,
        boxShadow: `0 0 ${glowIntensity * 20}px ${color}20`,
      }}
    >
      <span style={{ fontSize: s.iconSize }}>{icon}</span>
      <span
        style={{
          color: color,
          fontSize: s.fontSize,
          fontWeight: 600,
          letterSpacing: '-0.01em',
        }}
      >
        {label}
      </span>
    </div>
  );
};

// Logo grid animation
interface LogoItemProps {
  name: string;
  icon: string;
  delay: number;
  color: string;
}

export const LogoItem: React.FC<LogoItemProps> = ({
  name,
  icon,
  delay,
  color,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const scale = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 15, stiffness: 150, mass: 0.3 },
  });
  
  const opacity = interpolate(effectiveFrame, [0, 8], [0, 1], {
    extrapolateRight: 'clamp',
  });
  
  const glowPulse = Math.sin(effectiveFrame * 0.05) * 0.2 + 0.8;
  
  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale})`,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 8,
      }}
    >
      <div
        style={{
          fontSize: 36,
          filter: `drop-shadow(0 0 ${glowPulse * 12}px ${color}60)`,
        }}
      >
        {icon}
      </div>
      <span
        style={{
          color: 'rgba(255, 255, 255, 0.7)',
          fontSize: 11,
          fontWeight: 500,
          letterSpacing: '0.05em',
          textTransform: 'uppercase',
        }}
      >
        {name}
      </span>
    </div>
  );
};

// Animated counter
interface AnimatedCounterProps {
  from: number;
  to: number;
  delay?: number;
  duration?: number;
  suffix?: string;
  prefix?: string;
  color?: string;
  fontSize?: number;
}

export const AnimatedCounter: React.FC<AnimatedCounterProps> = ({
  from,
  to,
  delay = 0,
  duration = 60,
  suffix = '',
  prefix = '',
  color = '#ffffff',
  fontSize = 48,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  const progress = interpolate(effectiveFrame, [0, duration], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });
  
  const value = Math.round(interpolate(progress, [0, 1], [from, to]));
  
  return (
    <span
      style={{
        color,
        fontSize,
        fontWeight: 700,
        fontVariantNumeric: 'tabular-nums',
      }}
    >
      {prefix}{value}{suffix}
    </span>
  );
};

// Smooth line animation for drawing effects
export function useLineProgress(delay: number, duration: number) {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);
  
  return interpolate(effectiveFrame, [0, duration], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.cubic),
  });
}
