import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  spring,
  interpolate,
  Easing,
} from 'remotion';
import { typography } from '../fonts';

interface TitleProps {
  title: string;
  subtitle?: string;
  delay?: number;
  color?: string;
  fontSize?: number;
  glow?: boolean;
  gradient?: string;
}

export const Title: React.FC<TitleProps> = ({
  title,
  subtitle,
  delay = 0,
  color = '#ffffff',
  fontSize = 80,
  glow = true,
  gradient,
}) => {
  const frame = useCurrentFrame();

  // Apple 风格的 Spring 动画参数
  const springConfig = {
    damping: 200,
    stiffness: 100,
    mass: 0.6,
  };

  const effectiveFrame = Math.max(0, frame - delay);

  // 标题不透明度：平滑淡入
  const titleOpacity = spring({
    frame: effectiveFrame,
    fps: 30,
    config: springConfig,
  });

  // 标题 Y 轴位置：优雅向上滑动
  const titleY = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: {
        damping: 200,
        stiffness: 80,
        mass: 0.7,
      },
    }),
    [0, 1],
    [50, 0]
  );

  // 标题缩放：微妙的放大效果
  const titleScale = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: {
        damping: 200,
        stiffness: 60,
        mass: 1,
      },
    }),
    [0, 1],
    [0.92, 1]
  );

  // 光晕效果：动态脉冲
  const glowIntensity = glow
    ? interpolate(effectiveFrame, [0, 90], [0, 1], {
        extrapolateRight: 'clamp',
        easing: Easing.out(Easing.ease),
      })
    : 0;

  // 副标题延迟出现
  const subtitleDelay = 15;
  const subtitleFrame = Math.max(0, effectiveFrame - subtitleDelay);
  const subtitleOpacity = interpolate(subtitleFrame, [0, 25], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });
  const subtitleY = interpolate(subtitleFrame, [0, 25], [20, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  // 渐变样式
  const gradientStyle = gradient
    ? {
        background: gradient,
        WebkitBackgroundClip: 'text' as const,
        WebkitTextFillColor: 'transparent' as const,
        backgroundClip: 'text' as const,
      }
    : { color };

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
      }}
    >
      <h1
        style={{
          ...typography.title,
          fontSize,
          ...gradientStyle,
          opacity: titleOpacity,
          transform: `translateY(${titleY}px) scale(${titleScale})`,
          textAlign: 'center',
          padding: '0 100px',
          lineHeight: 1.1,
          textShadow: glow && !gradient
            ? `0 0 ${glowIntensity * 40}px rgba(255, 255, 255, ${glowIntensity * 0.25})`
            : 'none',
          filter: glow && !gradient ? `brightness(${1 + glowIntensity * 0.08})` : 'none',
        }}
      >
        {title}
      </h1>
      {subtitle && (
        <p
          style={{
            color: 'rgba(255, 255, 255, 0.6)',
            fontSize: fontSize * 0.3,
            ...typography.subtitle,
            opacity: subtitleOpacity,
            marginTop: 20,
            transform: `translateY(${subtitleY}px)`,
            textAlign: 'center',
            padding: '0 100px',
            lineHeight: 1.5,
          }}
        >
          {subtitle}
        </p>
      )}
    </AbsoluteFill>
  );
};
