import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {TerminalAnimation} from '../components/TerminalAnimation';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

export const SolutionScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera: zoom out to reveal - triumphant feeling
  const cameraZoom = interpolate(frame, [0, 60], [1.12, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  // Gentle upward drift
  const cameraDriftY = interpolate(frame, [0, 250], [10, -5], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  const titleOpacity = spring({
    frame,
    fps: 30,
    config: {
      damping: 15,
      stiffness: 80,
      mass: 0.5,
    },
  });

  const titleY = interpolate(
    spring({
      frame,
      fps: 30,
      config: {
        damping: 15,
        stiffness: 100,
        mass: 0.6,
      },
    }),
    [0, 1],
    [40, 0]
  );

  const titleScale = interpolate(
    spring({
      frame,
      fps: 30,
      config: {
        damping: 15,
        stiffness: 60,
        mass: 0.8,
      },
    }),
    [0, 1],
    [0.9, 1]
  );

  // Terminal lines with typing animation
  const terminalLines = [
    { type: 'command' as const, text: 'vx npx create-react-app my-app', delay: 0, typeSpeed: 1.2 },
    { type: 'output' as const, text: '⚡ Installing Node.js v20.11.0...', delay: 3 },
    { type: 'progress' as const, text: '', delay: 3, typeSpeed: 0.6 },
    { type: 'output' as const, text: '✓ Node.js ready!', delay: 3 },
    { type: 'output' as const, text: '✓ Running npx create-react-app...', delay: 8 },
    { type: 'command' as const, text: 'vx go run main.go', delay: 25, typeSpeed: 1.2 },
    { type: 'output' as const, text: '⚡ Installing Go v1.23.4...', delay: 3 },
    { type: 'progress' as const, text: '', delay: 3, typeSpeed: 0.5 },
    { type: 'output' as const, text: '✓ Done! Output: Hello, VX!', delay: 3 },
  ];

  // Benefits with staggered animation
  const benefits = [
    { text: '零学习成本', icon: '🎯' },
    { text: '零配置', icon: '⚙️' },
    { text: '零冲突', icon: '✨' },
  ];

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Floating provider logos - like in space, green theme */}
      <FloatingLogos 
        count={5} 
        baseSize={55} 
        baseBlur={5} 
        opacity={0.7}
        seed={3}
        logos={['Node.js', 'Go', 'Python', 'Rust', 'Deno']}
      />
      
      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '40px 80px',
          transform: `scale(${cameraZoom}) translateY(${cameraDriftY}px)`,
        }}
      >
        {/* Title */}
        <div
          style={{
            opacity: titleOpacity,
            transform: `translateY(${titleY}px) scale(${titleScale})`,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            marginBottom: 30,
          }}
        >
          <h1
            style={{
              ...typography.title,
              fontSize: 56,
              textAlign: 'center',
              background: 'linear-gradient(135deg, #86efac 0%, #22c55e 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
            }}
          >
            使用 VX：开箱即用
          </h1>
          <p
            style={{
              ...typography.subtitle,
                  color: 'rgba(255, 255, 255, 0.7)',
              fontSize: 20,
              marginTop: 8,
              opacity: interpolate(frame, [10, 30], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }),
            }}
          >
            相同命令，零配置
          </p>
        </div>
        
        {/* Animated terminal */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            width: '100%',
            maxWidth: 800,
          }}
        >
          <TerminalAnimation
            lines={terminalLines}
            title="VX"
            delay={25}
            fontSize={15}
          />
        </div>

        {/* Bottom benefits with staggered animation */}
        <div
          style={{
            marginTop: 35,
            display: 'flex',
            gap: 30,
          }}
        >
          {benefits.map((benefit, index) => {
            const pointDelay = 200 + index * 15;
            const pointOpacity = interpolate(frame, [pointDelay, pointDelay + 20], [0, 1], { 
              extrapolateLeft: 'clamp', 
              extrapolateRight: 'clamp' 
            });
            const pointY = interpolate(frame, [pointDelay, pointDelay + 20], [15, 0], { 
              extrapolateLeft: 'clamp', 
              extrapolateRight: 'clamp' 
            });
            const pointScale = interpolate(frame, [pointDelay, pointDelay + 20], [0.9, 1], { 
              extrapolateLeft: 'clamp', 
              extrapolateRight: 'clamp' 
            });

            return (
              <div
                key={index}
                style={{
                  opacity: pointOpacity,
                  transform: `translateY(${pointY}px) scale(${pointScale})`,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  padding: '10px 18px',
                  backgroundColor: 'rgba(34, 197, 94, 0.08)',
                  borderRadius: 100,
                  border: '1px solid rgba(34, 197, 94, 0.2)',
                }}
              >
                <span style={{ fontSize: 18 }}>{benefit.icon}</span>
                <span
                  style={{
                    ...typography.body,
                    color: 'rgba(255, 255, 255, 0.7)',
                    fontSize: 14,
                    fontWeight: 500,
                  }}
                >
                  {benefit.text}
                </span>
              </div>
            );
          })}
        </div>
      </AbsoluteFill>
      
      {/* Subtle vignette */}
      <AbsoluteFill
        style={{
          background: 'radial-gradient(ellipse at center, transparent 65%, rgba(0,0,0,0.12) 100%)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
