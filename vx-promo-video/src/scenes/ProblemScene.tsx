import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {TerminalAnimation} from '../components/TerminalAnimation';
import {FadeIn} from '../components/AnimationUtils';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

export const ProblemScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera: slow zoom in effect - creates tension
  const cameraZoom = interpolate(frame, [0, 300], [1, 1.08], {
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.ease),
  });

  // Subtle shake for frustration feeling
  const shakeX = frame > 180 ? Math.sin(frame * 0.3) * 2 : 0;
  const shakeY = frame > 180 ? Math.cos(frame * 0.4) * 1 : 0;

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

  // Terminal lines showing painful traditional setup
  const terminalLines = [
    { type: 'comment' as const, text: '配置新开发环境...', delay: 0 },
    { type: 'command' as const, text: 'curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash', delay: 8, typeSpeed: 0.6 },
    { type: 'output' as const, text: '=> Downloading nvm from git...', delay: 3 },
    { type: 'progress' as const, text: '', delay: 3, typeSpeed: 0.4 },
    { type: 'output' as const, text: '=> Please restart your terminal or run: source ~/.bashrc', delay: 3 },
    { type: 'command' as const, text: 'nvm install 20', delay: 15, typeSpeed: 1.2 },
    { type: 'output' as const, text: 'Downloading node v20.11.0...', delay: 3 },
    { type: 'progress' as const, text: '', delay: 3, typeSpeed: 0.3 },
    { type: 'comment' as const, text: '接下来还要安装 Python, Go, Rust...', delay: 8 },
    { type: 'comment' as const, text: '处理 PATH 冲突...', delay: 8 },
    { type: 'comment' as const, text: '几小时后... 😩', delay: 8 },
  ];

  // Bottom message animation with stagger
  const painPoints = [
    { text: '多种安装器', icon: '📦' },
    { text: 'PATH 冲突', icon: '⚠️' },
    { text: '漫长配置', icon: '⏰' },
  ];

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Floating provider logos - like in space */}
      <FloatingLogos 
        count={4} 
        baseSize={50} 
        baseBlur={5} 
        opacity={0.6}
        seed={2}
        logos={['npm', 'Yarn', 'pnpm', 'uv']}
      />
      
      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '40px 80px',
          transform: `scale(${cameraZoom}) translate(${shakeX}px, ${shakeY}px)`,
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
              background: 'linear-gradient(135deg, #fca5a5 0%, #ef4444 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
            }}
          >
            传统方式
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
            从零开始搭建开发环境
          </p>
        </div>
        
        {/* Animated terminal */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            width: '100%',
            maxWidth: 900,
          }}
        >
          <TerminalAnimation
            lines={terminalLines}
            title="Terminal"
            delay={20}
            fontSize={14}
          />
        </div>

        {/* Bottom pain points with staggered animation */}
        <div
          style={{
            marginTop: 35,
            display: 'flex',
            gap: 40,
          }}
        >
          {painPoints.map((point, index) => {
            const pointDelay = 220 + index * 15;
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
                  backgroundColor: 'rgba(239, 68, 68, 0.08)',
                  borderRadius: 100,
                  border: '1px solid rgba(239, 68, 68, 0.2)',
                }}
              >
                <span style={{ fontSize: 18 }}>{point.icon}</span>
                <span
                  style={{
                    ...typography.body,
                    color: 'rgba(255, 255, 255, 0.7)',
                    fontSize: 14,
                    fontWeight: 500,
                  }}
                >
                  {point.text}
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
