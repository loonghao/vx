import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {ParticleField} from '../components/ParticleField';
import {RadialPulse, LightLeak} from '../components/Transitions';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

export const IntroScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera zoom out effect - dramatic reveal
  const cameraZoom = interpolate(frame, [0, 90], [1.2, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  // Subtle camera drift
  const cameraDriftX = Math.sin(frame * 0.02) * 5;
  const cameraDriftY = Math.cos(frame * 0.015) * 3;

  // Logo animation - more dramatic entrance
  const logoScale = interpolate(
    spring({
      frame,
      fps: 30,
      config: {
        damping: 180,
        stiffness: 80,
        mass: 0.8,
      },
    }),
    [0, 1],
    [0.6, 1]
  );

  const logoOpacity = spring({
    frame,
    fps: 30,
    config: {
      damping: 200,
      stiffness: 100,
      mass: 0.5,
    },
  });

  // Logo Y position - float up
  const logoY = interpolate(
    spring({
      frame,
      fps: 30,
      config: {
        damping: 200,
        stiffness: 60,
        mass: 1,
      },
    }),
    [0, 1],
    [30, 0]
  );

  const titleOpacity = spring({
    frame: Math.max(0, frame - 20),
    fps: 30,
    config: {
      damping: 200,
      stiffness: 80,
      mass: 0.6,
    },
  });

  const titleY = interpolate(
    spring({
      frame: Math.max(0, frame - 20),
      fps: 30,
      config: {
        damping: 200,
        stiffness: 80,
        mass: 0.7,
      },
    }),
    [0, 1],
    [40, 0]
  );

  const subtitleOpacity = interpolate(
    Math.max(0, frame - 40),
    [0, 25],
    [0, 1],
    { extrapolateRight: 'clamp', easing: Easing.out(Easing.ease) }
  );

  const subtitleY = interpolate(
    Math.max(0, frame - 40),
    [0, 25],
    [20, 0],
    { extrapolateRight: 'clamp', easing: Easing.out(Easing.ease) }
  );

  // Dynamic glow effect
  const glowIntensity = interpolate(frame, [0, 90], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  // Subtle breathing animation for logo
  const breathe = Math.sin(frame / 30) * 0.02;
  
  // Rotating glow halo
  const haloRotation = frame * 0.5;

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Particle field for depth */}
      <ParticleField count={60} variant="sparkle" intensity={0.5} />
      
      {/* Floating provider logos - like in space */}
      <FloatingLogos 
        count={6} 
        baseSize={60} 
        baseBlur={4} 
        seed={1}
        logos={['Node.js', 'Python', 'Go', 'Rust', 'Java', 'Docker']}
      />
      
      {/* Radial pulse on logo appear */}
      <RadialPulse delay={5} color="#67e8f9" maxSize={600} />
      
      {/* Light leak effect */}
      <LightLeak delay={30} duration={40} color="#a78bfa" position="right" />
      
      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'center',
          alignItems: 'center',
          gap: 20,
          transform: `scale(${cameraZoom}) translate(${cameraDriftX}px, ${cameraDriftY}px)`,
        }}
      >
        {/* Rotating glow halo behind logo */}
        <div
          style={{
            position: 'absolute',
            width: 400,
            height: 400,
            background: `conic-gradient(from ${haloRotation}deg, transparent, rgba(103, 232, 249, ${glowIntensity * 0.15}), transparent, rgba(167, 139, 250, ${glowIntensity * 0.15}), transparent)`,
            borderRadius: '50%',
            filter: 'blur(60px)',
            opacity: glowIntensity,
          }}
        />
        
        {/* VX Logo - Premium look */}
        <div
          style={{
            ...typography.title,
            opacity: logoOpacity,
            transform: `scale(${logoScale + breathe}) translateY(${logoY}px)`,
            fontSize: 160,
            fontWeight: 800,
            background: 'linear-gradient(135deg, #67e8f9 0%, #a78bfa 50%, #f472b6 100%)',
            backgroundClip: 'text',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            letterSpacing: '-0.06em',
            filter: `drop-shadow(0 0 ${glowIntensity * 50}px rgba(103, 232, 249, 0.4)) drop-shadow(0 0 ${glowIntensity * 25}px rgba(167, 139, 250, 0.3))`,
            position: 'relative',
            zIndex: 1,
          }}
        >
          vx
        </div>

        {/* Tagline */}
        <h1
          style={{
            ...typography.title,
            fontSize: 52,
            opacity: titleOpacity,
            transform: `translateY(${titleY}px)`,
            textAlign: 'center',
            marginTop: 8,
            background: 'linear-gradient(135deg, #ffffff 0%, rgba(255, 255, 255, 0.8) 100%)',
            backgroundClip: 'text',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            position: 'relative',
            zIndex: 1,
          }}
        >
          通用开发工具管理器
        </h1>

        {/* Subtitle */}
        <p
          style={{
            ...typography.subtitle,
            color: 'rgba(255, 255, 255, 0.7)',
            fontSize: 24,
            opacity: subtitleOpacity,
            transform: `translateY(${subtitleY}px)`,
            textAlign: 'center',
            maxWidth: 700,
            lineHeight: 1.5,
            fontWeight: 400,
            position: 'relative',
            zIndex: 1,
          }}
        >
          一行命令，所有工具，零配置
        </p>
      </AbsoluteFill>
      
      {/* Vignette */}
      <AbsoluteFill
        style={{
          background: 'radial-gradient(ellipse at center, transparent 55%, rgba(0,0,0,0.2) 100%)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
