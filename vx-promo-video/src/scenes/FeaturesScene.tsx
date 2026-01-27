import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

interface FeatureItemProps {
  icon: string;
  title: string;
  description: string;
  delay: number;
  color: string;
  index: number;
}

const FeatureItem: React.FC<FeatureItemProps> = ({icon, title, description, delay, color, index}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const opacity = spring({
    frame: effectiveFrame,
    fps: 30,
    config: {
      damping: 150,
      stiffness: 120,
      mass: 0.4,
    },
  });

  const translateY = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: {
        damping: 150,
        stiffness: 120,
        mass: 0.5,
      },
    }),
    [0, 1],
    [25, 0]
  );

  const scale = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: {
        damping: 150,
        stiffness: 130,
        mass: 0.4,
      },
    }),
    [0, 1],
    [0.94, 1]
  );

  const glowIntensity = interpolate(effectiveFrame, [0, 30], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  return (
    <div
      style={{
        opacity,
        transform: `translateY(${translateY}px) scale(${scale})`,
        display: 'flex',
        alignItems: 'center',
        gap: 18,
        padding: '20px 26px',
        backgroundColor: 'rgba(255, 255, 255, 0.03)',
        borderRadius: 16,
        border: `1px solid rgba(255, 255, 255, ${0.06 + glowIntensity * 0.03})`,
        boxShadow: `
          0 12px 32px -8px rgba(0, 0, 0, 0.4),
          0 0 ${glowIntensity * 20}px ${color}15,
          inset 0 1px 0 rgba(255, 255, 255, 0.05)
        `,
        width: '100%',
      }}
    >
      <div
        style={{
          fontSize: 32,
          filter: `drop-shadow(0 0 ${glowIntensity * 15}px ${color}60)`,
          opacity: 0.95,
          flexShrink: 0,
        }}
      >
        {icon}
      </div>
      <div style={{ flex: 1 }}>
        <h3
          style={{
            ...typography.title,
            color: '#ffffff',
            fontSize: 18,
            marginBottom: 5,
            fontWeight: 600,
          }}
        >
          {title}
        </h3>
        <p
          style={{
            ...typography.body,
            color: 'rgba(255, 255, 255, 0.7)',
            fontSize: 13,
            lineHeight: 1.45,
            margin: 0,
          }}
        >
          {description}
        </p>
      </div>
    </div>
  );
};

export const FeaturesScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera: start wide, subtle push in
  const cameraZoom = interpolate(frame, [0, 180], [0.95, 1.02], {
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.ease),
  });

  // Subtle pan effect
  const cameraPanX = interpolate(frame, [0, 180], [-8, 8], {
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.ease),
  });

  const titleOpacity = spring({
    frame,
    fps: 30,
    config: {
      damping: 150,
      stiffness: 120,
      mass: 0.4,
    },
  });

  const titleY = interpolate(
    spring({
      frame,
      fps: 30,
      config: {
        damping: 150,
        stiffness: 120,
        mass: 0.5,
      },
    }),
    [0, 1],
    [30, 0]
  );

  const features = [
    {
      icon: '⚡',
      title: '零学习成本',
      description: '使用你已经熟悉的命令，只需加上 vx 前缀',
      color: '#fbbf24',
    },
    {
      icon: '🔧',
      title: '首次使用自动安装',
      description: '工具在需要时自动安装，无需手动配置',
      color: '#34d399',
    },
    {
      icon: '🤖',
      title: 'AI 原生',
      description: '为 Claude、Cursor 和 MCP 服务器而生',
      color: '#a78bfa',
    },
    {
      icon: '📦',
      title: '隔离干净',
      description: '无 PATH 冲突，无系统污染',
      color: '#60a5fa',
    },
    {
      icon: '🌍',
      title: '跨平台',
      description: '相同命令，随处可用',
      color: '#f472b6',
    },
    {
      icon: '👥',
      title: '团队友好',
      description: '共享 vx.toml，统一版本',
      color: '#f87171',
    },
  ];

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Floating provider logos - like in space */}
      <FloatingLogos 
        count={5} 
        baseSize={50} 
        baseBlur={5} 
        opacity={0.5}
        seed={4}
        logos={['Bun', 'Ruby', 'Java', 'Docker', 'Git']}
      />

      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '50px 100px',
          transform: `scale(${cameraZoom}) translateX(${cameraPanX}px)`,
        }}
      >
        {/* Title with gradient */}
        <h1
          style={{
            ...typography.title,
            fontSize: 52,
            textAlign: 'center',
            marginBottom: 45,
            opacity: titleOpacity,
            transform: `translateY(${titleY}px)`,
            background: 'linear-gradient(135deg, #ffffff 0%, #67e8f9 100%)',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            backgroundClip: 'text',
          }}
        >
          为什么开发者喜欢 VX
        </h1>

        {/* Features grid - centered 3x2 layout */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: 20,
            width: '100%',
            maxWidth: 1100,
          }}
        >
          {features.map((feature, index) => (
            <FeatureItem
              key={index}
              icon={feature.icon}
              title={feature.title}
              description={feature.description}
              delay={12 + index * 6}
              color={feature.color}
              index={index}
            />
          ))}
        </div>
      </AbsoluteFill>
      
      {/* Subtle vignette */}
      <AbsoluteFill
        style={{
          background: 'radial-gradient(ellipse at center, transparent 65%, rgba(0,0,0,0.15) 100%)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
