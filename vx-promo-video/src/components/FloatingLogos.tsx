import React from 'react';
import {useCurrentFrame, interpolate} from 'remotion';
import {ProviderLogos} from './ProviderLogos';

// All available provider logo names
const ALL_LOGOS = Object.keys(ProviderLogos) as (keyof typeof ProviderLogos)[];

// Predefined positions for floating logos to ensure good distribution
const LOGO_POSITIONS = [
  {x: 8, y: 15},
  {x: 92, y: 12},
  {x: 5, y: 85},
  {x: 95, y: 80},
  {x: 15, y: 50},
  {x: 85, y: 55},
  {x: 50, y: 8},
  {x: 45, y: 92},
  {x: 25, y: 25},
  {x: 75, y: 75},
  {x: 30, y: 70},
  {x: 70, y: 30},
];

interface FloatingLogoProps {
  name: keyof typeof ProviderLogos;
  x: number;
  y: number;
  size: number;
  delay: number;
  blur: number;
  seed: number;
}

// Single floating logo component
const FloatingLogo: React.FC<FloatingLogoProps> = ({name, x, y, size, delay, blur, seed}) => {
  const frame = useCurrentFrame();
  const Logo = ProviderLogos[name];
  
  if (!Logo) return null;
  
  // Delayed fade in - like emerging from the void of space
  const opacity = interpolate(
    frame - delay,
    [0, 60, 90],
    [0, 0.25, 0.4],
    {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'}
  );
  
  // Gentle floating motion - different for each logo based on seed
  const floatY = Math.sin((frame + seed * 47) / 55) * 12;
  const floatX = Math.cos((frame + seed * 31) / 70) * 6;
  
  // Subtle scale breathing - like gentle pulsing in space
  const breathe = 1 + Math.sin((frame + seed * 23) / 45) * 0.04;
  
  // Very subtle rotation - slow drift
  const rotation = Math.sin((frame + seed * 67) / 100) * 4;
  
  // Pulsing glow - ethereal space effect
  const glowOpacity = 0.2 + Math.sin((frame + seed * 13) / 35) * 0.1;
  
  return (
    <div
      style={{
        position: 'absolute',
        left: `${x}%`,
        top: `${y}%`,
        transform: `translate(-50%, -50%) translate(${floatX}px, ${floatY}px) scale(${breathe}) rotate(${rotation}deg)`,
        opacity,
        filter: `blur(${blur}px)`,
        pointerEvents: 'none',
      }}
    >
      {/* Ethereal glow behind logo */}
      <div
        style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: size * 2,
          height: size * 2,
          background: 'radial-gradient(circle, rgba(103, 232, 249, 0.25) 0%, rgba(167, 139, 250, 0.15) 40%, transparent 70%)',
          borderRadius: '50%',
          opacity: glowOpacity,
          filter: `blur(${blur + 15}px)`,
        }}
      />
      <Logo size={size} />
    </div>
  );
};

interface FloatingLogosProps {
  // Number of logos to display (default: 6)
  count?: number;
  // Base size for logos (default: 50)
  baseSize?: number;
  // Base blur amount (default: 4)
  baseBlur?: number;
  // Base opacity multiplier (default: 1)
  opacity?: number;
  // Specific logos to use (if not specified, uses random selection)
  logos?: (keyof typeof ProviderLogos)[];
  // Random seed for consistent positioning
  seed?: number;
}

// Main component that renders multiple floating logos
export const FloatingLogos: React.FC<FloatingLogosProps> = ({
  count = 6,
  baseSize = 50,
  baseBlur = 4,
  opacity = 1,
  logos,
  seed = 0,
}) => {
  // Select logos to display
  const selectedLogos = React.useMemo(() => {
    if (logos && logos.length > 0) {
      return logos;
    }
    // Use a seeded shuffle for consistent results
    const shuffled = [...ALL_LOGOS].sort((a, b) => {
      const hashA = a.charCodeAt(0) + seed;
      const hashB = b.charCodeAt(0) + seed;
      return hashA - hashB;
    });
    return shuffled.slice(0, count);
  }, [logos, count, seed]);
  
  // Generate logo configurations
  const logoConfigs = React.useMemo(() => {
    return selectedLogos.map((name, index) => {
      const position = LOGO_POSITIONS[index % LOGO_POSITIONS.length];
      // Add some variation based on index and seed
      const variation = ((index + seed) % 5) / 5;
      
      return {
        name,
        x: position.x + (variation * 6 - 3),
        y: position.y + (variation * 6 - 3),
        size: baseSize + (variation * 20 - 10),
        delay: index * 12 + seed * 3,
        blur: baseBlur + (variation * 2),
        seed: index + seed,
      };
    });
  }, [selectedLogos, baseSize, baseBlur, seed]);
  
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        overflow: 'hidden',
        opacity,
        pointerEvents: 'none',
      }}
    >
      {logoConfigs.map((config, index) => (
        <FloatingLogo key={`${config.name}-${index}`} {...config} />
      ))}
    </div>
  );
};
