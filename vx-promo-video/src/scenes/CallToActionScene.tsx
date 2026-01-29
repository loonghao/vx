import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
  Easing,
  Sequence,
} from 'remotion';
import {Background} from '../components/Background';
import {TerminalAnimation} from '../components/TerminalAnimation';
import {ParticleField} from '../components/ParticleField';
import {PulseGlow} from '../components/GlowEffects';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

export const CallToActionScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera: dramatic zoom out to reveal everything
  const cameraZoom = interpolate(frame, [0, 80], [1.15, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  // Epic slow push in at the end
  const endZoom = interpolate(frame, [150, 220], [1, 1.05], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.ease),
  });

  // Combined zoom
  const finalZoom = frame < 150 ? cameraZoom : endZoom;

  // Smooth entrance animation
  const entrance = spring({
    frame,
    fps: 30,
    config: {
      damping: 15,
      stiffness: 60,
      mass: 0.8,
    },
  });

  const titleOpacity = interpolate(entrance, [0, 1], [0, 1]);
  const titleY = interpolate(entrance, [0, 1], [50, 0]);
  const titleScale = interpolate(entrance, [0, 1], [0.9, 1]);

  // Subtle glow pulse
  const glowPulse = Math.sin(frame * 0.06) * 0.15 + 0.85;
  const glowIntensity = interpolate(frame, [0, 60], [0, 1], {
    extrapolateRight: 'clamp',
  });

  // Terminal lines - clean installation using winget
  const terminalLines = [
    {type: 'comment' as const, text: '安装 VX - 通用开发工具管理器', delay: 0},
    {type: 'command' as const, text: 'winget install vx', delay: 8, typeSpeed: 1.5},
    {type: 'output' as const, text: '正在下载 VX...', delay: 5},
    {type: 'progress' as const, text: '', delay: 3, typeSpeed: 0.3},
    {type: 'output' as const, text: '✓ VX 安装成功！', delay: 3},
    {type: 'output' as const, text: '✓ 自动检测: Node.js, Python, Go, Rust', delay: 5},
    {type: 'output' as const, text: '✓ 准备就绪！', delay: 5},
  ];

  // Website reveal animation
  const websiteReveal = interpolate(frame, [140, 165], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  const websiteY = interpolate(frame, [140, 165], [30, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  return (
    <AbsoluteFill>
      <Background variant="apple" />

      {/* Subtle particle background */}
      <Sequence from={0}>
        <ParticleField count={40} variant="sparkle" intensity={0.4} />
      </Sequence>

      {/* Soft glow */}
      <PulseGlow delay={0} color="#67e8f9" />

      {/* Floating provider logos - like in space */}
      <FloatingLogos 
        count={6} 
        baseSize={55} 
        baseBlur={4} 
        opacity={0.6}
        seed={6}
        logos={['Node.js', 'Python', 'Go', 'Rust', 'Deno', 'Bun']}
      />

      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'center',
          alignItems: 'center',
          padding: '40px 100px',
          transform: `scale(${finalZoom})`,
        }}
      >
        {/* Title */}
        <div
          style={{
            position: 'relative',
            marginBottom: 12,
          }}
        >
          {/* Glow behind text */}
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              width: 350,
              height: 120,
              background: `radial-gradient(ellipse, rgba(103, 232, 249, ${glowIntensity * glowPulse * 0.25}) 0%, transparent 70%)`,
              filter: 'blur(40px)',
              pointerEvents: 'none',
            }}
          />

          <h1
            style={{
              ...typography.title,
              fontSize: 80,
              textAlign: 'center',
              opacity: titleOpacity,
              transform: `translateY(${titleY}px) scale(${titleScale})`,
              background: `linear-gradient(135deg, 
                #ffffff 0%, 
                #67e8f9 40%, 
                #a78bfa 100%
              )`,
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
              filter: `drop-shadow(0 0 ${glowIntensity * 20}px rgba(103, 232, 249, 0.3))`,
              letterSpacing: '-0.02em',
              position: 'relative',
            }}
          >
            立即开始
          </h1>
        </div>

        {/* Tagline */}
        <p
          style={{
            ...typography.subtitle,
            color: 'rgba(255, 255, 255, 0.6)',
            fontSize: 24,
            textAlign: 'center',
            opacity: interpolate(frame, [20, 40], [0, 1], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            }),
            transform: `translateY(${interpolate(frame, [20, 40], [15, 0], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            })}px)`,
            marginBottom: 40,
            fontWeight: 500,
          }}
        >
          一行命令，统一管理
        </p>

        {/* Terminal */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            width: '100%',
            maxWidth: 800,
            opacity: interpolate(frame, [30, 50], [0, 1], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            }),
            transform: `translateY(${interpolate(frame, [30, 50], [20, 0], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            })}px)`,
          }}
        >
          <TerminalAnimation
            lines={terminalLines}
            title="PowerShell"
            delay={40}
            fontSize={15}
          />
        </div>

        {/* Website CTA */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: 10,
            marginTop: 45,
            opacity: websiteReveal,
            transform: `translateY(${websiteY}px)`,
          }}
        >
          <span
            style={{
              ...typography.body,
            color: 'rgba(255, 255, 255, 0.6)',
              fontSize: 16,
              letterSpacing: '0.1em',
              textTransform: 'uppercase',
            }}
          >
            访问官网
          </span>

          {/* VX.sh */}
          <div style={{position: 'relative'}}>
            {/* Subtle glow */}
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                width: 200,
                height: 70,
                background: `radial-gradient(ellipse, 
                  rgba(103, 232, 249, ${websiteReveal * glowPulse * 0.3}) 0%, 
                  transparent 70%
                )`,
                filter: 'blur(20px)',
                pointerEvents: 'none',
              }}
            />

            <span
              style={{
                ...typography.title,
                fontSize: 56,
                fontWeight: 800,
                background: `linear-gradient(135deg, #67e8f9 0%, #a78bfa 100%)`,
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
                backgroundClip: 'text',
                letterSpacing: '-0.01em',
                position: 'relative',
                filter: `drop-shadow(0 0 ${websiteReveal * 20}px rgba(103, 232, 249, 0.4))`,
              }}
            >
              vx.sh
            </span>
          </div>

          {/* Tagline */}
          <span
            style={{
              ...typography.body,
              color: 'rgba(255, 255, 255, 0.35)',
              fontSize: 14,
              letterSpacing: '0.12em',
              textTransform: 'uppercase',
              marginTop: 8,
              opacity: interpolate(frame, [170, 190], [0, 1], {
                extrapolateLeft: 'clamp',
                extrapolateRight: 'clamp',
              }),
            }}
          >
            立即体验
          </span>
        </div>
      </AbsoluteFill>

      {/* Vignette */}
      <AbsoluteFill
        style={{
          background: 'radial-gradient(ellipse at center, transparent 65%, rgba(0,0,0,0.18) 100%)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
