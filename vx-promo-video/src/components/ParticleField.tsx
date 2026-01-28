import React, {useMemo} from 'react';
import {AbsoluteFill, useCurrentFrame, interpolate, Easing} from 'remotion';
import {noise2D} from '@remotion/noise';

interface Particle {
  id: number;
  x: number;
  y: number;
  size: number;
  speed: number;
  opacity: number;
  hue: number;
  delay: number;
}

interface ParticleFieldProps {
  count?: number;
  variant?: 'default' | 'explosion' | 'converge' | 'sparkle';
  color?: string;
  intensity?: number;
}

export const ParticleField: React.FC<ParticleFieldProps> = ({
  count = 80,
  variant = 'default',
  intensity = 1,
}) => {
  const frame = useCurrentFrame();

  // Generate particles with deterministic randomness
  const particles = useMemo<Particle[]>(() => {
    const result: Particle[] = [];
    for (let i = 0; i < count; i++) {
      const seed = i * 12345;
      result.push({
        id: i,
        x: (Math.sin(seed) * 0.5 + 0.5) * 100,
        y: (Math.cos(seed * 2) * 0.5 + 0.5) * 100,
        size: 1 + (Math.sin(seed * 3) * 0.5 + 0.5) * 4,
        speed: 0.5 + (Math.cos(seed * 4) * 0.5 + 0.5) * 1.5,
        opacity: 0.3 + (Math.sin(seed * 5) * 0.5 + 0.5) * 0.7,
        hue: 180 + (Math.cos(seed * 6) * 0.5 + 0.5) * 80, // Cyan to purple range
        delay: Math.floor((Math.sin(seed * 7) * 0.5 + 0.5) * 60),
      });
    }
    return result;
  }, [count]);

  const renderParticle = (particle: Particle) => {
    const effectiveFrame = Math.max(0, frame - particle.delay);
    
    // Base animation
    const noiseX = noise2D('x' + particle.id, effectiveFrame * 0.008, 0) * 30;
    const noiseY = noise2D('y' + particle.id, effectiveFrame * 0.008, 0) * 30;

    let x = particle.x;
    let y = particle.y;
    let scale = 1;
    let opacity = particle.opacity * intensity;

    switch (variant) {
      case 'explosion': {
        // Particles explode from center
        const progress = interpolate(effectiveFrame, [0, 60], [0, 1], {
          extrapolateRight: 'clamp',
          easing: Easing.out(Easing.cubic),
        });
        const angle = (particle.id / count) * Math.PI * 2;
        const distance = progress * 60;
        x = 50 + Math.cos(angle) * distance + noiseX * 0.3;
        y = 50 + Math.sin(angle) * distance + noiseY * 0.3;
        scale = interpolate(progress, [0, 0.3, 1], [0, 1.5, 0.8]);
        opacity *= interpolate(progress, [0, 0.2, 1], [0, 1, 0.6]);
        break;
      }
      case 'converge': {
        // Particles converge to center then explode
        const phase1 = interpolate(effectiveFrame, [0, 45], [0, 1], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.cubic),
        });
        const phase2 = interpolate(effectiveFrame, [45, 90], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.out(Easing.cubic),
        });
        
        const angle = (particle.id / count) * Math.PI * 2;
        if (effectiveFrame < 45) {
          // Converge
          const startDist = 50;
          const endDist = 5;
          const distance = interpolate(phase1, [0, 1], [startDist, endDist]);
          x = 50 + Math.cos(angle) * distance;
          y = 50 + Math.sin(angle) * distance;
          scale = interpolate(phase1, [0, 1], [0.5, 1.5]);
          opacity *= interpolate(phase1, [0, 1], [0.3, 1]);
        } else {
          // Explode
          const distance = 5 + phase2 * 70;
          x = 50 + Math.cos(angle) * distance + noiseX * 0.5;
          y = 50 + Math.sin(angle) * distance + noiseY * 0.5;
          scale = interpolate(phase2, [0, 1], [1.5, 0.3]);
          opacity *= interpolate(phase2, [0, 0.5, 1], [1, 0.8, 0]);
        }
        break;
      }
      case 'sparkle': {
        // Twinkling stars
        x = particle.x + noiseX * 0.5;
        y = particle.y + noiseY * 0.5;
        const twinkle = Math.sin(effectiveFrame * 0.2 + particle.id) * 0.5 + 0.5;
        scale = 0.5 + twinkle * 1;
        opacity *= twinkle;
        break;
      }
      default: {
        // Floating particles
        x = particle.x + noiseX;
        y = particle.y + noiseY + effectiveFrame * particle.speed * 0.1;
        y = y % 110; // Wrap around
        if (y < 0) y += 100;
        break;
      }
    }

    // Fade in
    const fadeIn = interpolate(effectiveFrame, [0, 20], [0, 1], {
      extrapolateRight: 'clamp',
    });

    return (
      <div
        key={particle.id}
        style={{
          position: 'absolute',
          left: `${x}%`,
          top: `${y}%`,
          width: particle.size * scale,
          height: particle.size * scale,
          borderRadius: '50%',
          backgroundColor: `hsla(${particle.hue}, 85%, 70%, ${opacity * fadeIn})`,
          boxShadow: `0 0 ${particle.size * 3}px hsla(${particle.hue}, 85%, 70%, ${opacity * fadeIn * 0.5})`,
          transform: 'translate(-50%, -50%)',
          pointerEvents: 'none',
        }}
      />
    );
  };

  return (
    <AbsoluteFill style={{overflow: 'hidden'}}>
      {particles.map(renderParticle)}
    </AbsoluteFill>
  );
};
