import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  Easing,
} from 'remotion';

interface BackgroundProps {
  color?: string;
  variant?: 'dark' | 'light' | 'gradient' | 'apple' | 'flow' | 'epic';
}

export const Background: React.FC<BackgroundProps> = ({
  color = '#0f172a',
  variant = 'dark',
}) => {
  const frame = useCurrentFrame();

  const renderBackground = () => {
    switch (variant) {
      case 'light':
        return <AbsoluteFill style={{backgroundColor: '#f8fafc'}} />;
      case 'gradient':
        return (
          <AbsoluteFill
            style={{
              background: `linear-gradient(135deg, #667eea 0%, #764ba2 100%)`,
            }}
          />
        );
      case 'apple':
        // Apple 风格渐变：极致深邃的黑色背景，配合微妙的动态光晕
        const appleGradientAngle = interpolate(frame, [0, 600], [140, 220], {
          extrapolateRight: 'clamp',
          extrapolateLeft: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        // 第一个光晕 - 蓝紫色
        const glow1X = interpolate(frame, [0, 600], [25, 75], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        const glow1Y = interpolate(frame, [0, 600], [20, 50], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        // 第二个光晕 - 青色
        const glow2X = interpolate(frame, [0, 600], [70, 30], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        const glow2Y = interpolate(frame, [0, 600], [60, 30], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        const glowOpacity = interpolate(frame, [0, 90, 300], [0, 0.25, 0.18], {
          extrapolateRight: 'clamp',
          easing: Easing.out(Easing.ease),
        });

        return (
          <AbsoluteFill>
            {/* 主背景：深邃但不过暗的背景 */}
            <AbsoluteFill
              style={{
                background: `linear-gradient(${appleGradientAngle}deg,
                  #0a0a12 0%,
                  #0c0c16 15%,
                  #0e0e1a 30%,
                  #10101e 50%,
                  #0e0e1a 70%,
                  #0c0c16 85%,
                  #0a0a12 100%
                )`,
              }}
            />
            
            {/* 微妙的噪点纹理层 */}
            <AbsoluteFill
              style={{
                opacity: 0.02,
                backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.7' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")`,
                mixBlendMode: 'overlay',
              }}
            />
            
            {/* 动态光晕效果 1 - 蓝紫色 */}
            <div
              style={{
                position: 'absolute',
                top: `${glow1Y}%`,
                left: `${glow1X}%`,
                width: '70%',
                height: '70%',
                transform: 'translate(-50%, -50%)',
                background: `radial-gradient(ellipse at center, 
                  rgba(139, 92, 246, ${glowOpacity}) 0%, 
                  rgba(99, 102, 241, ${glowOpacity * 0.5}) 25%,
                  transparent 60%
                )`,
                filter: 'blur(100px)',
                pointerEvents: 'none',
              }}
            />
            
            {/* 动态光晕效果 2 - 青色 */}
            <div
              style={{
                position: 'absolute',
                top: `${glow2Y}%`,
                left: `${glow2X}%`,
                width: '50%',
                height: '50%',
                transform: 'translate(-50%, -50%)',
                background: `radial-gradient(ellipse at center, 
                  rgba(103, 232, 249, ${glowOpacity * 0.8}) 0%, 
                  rgba(56, 189, 248, ${glowOpacity * 0.4}) 25%,
                  transparent 60%
                )`,
                filter: 'blur(80px)',
                pointerEvents: 'none',
              }}
            />
            
            {/* 顶部渐变遮罩 - 更微妙 */}
            <AbsoluteFill
              style={{
                background: 'linear-gradient(180deg, rgba(0,0,0,0.15) 0%, transparent 20%)',
              }}
            />
            
            {/* 底部渐变遮罩 - 更微妙 */}
            <AbsoluteFill
              style={{
                background: 'linear-gradient(0deg, rgba(0,0,0,0.12) 0%, transparent 15%)',
              }}
            />
            
            {/* 边缘暗角 - 减弱 */}
            <AbsoluteFill
              style={{
                background: 'radial-gradient(ellipse at center, transparent 50%, rgba(0,0,0,0.2) 100%)',
              }}
            />
          </AbsoluteFill>
        );
      case 'flow':
        // 流动渐变效果：动态颜色流动
        const flowOffset = interpolate(frame, [0, 180], [0, 360], {
          extrapolateRight: 'clamp',
          extrapolateLeft: 'clamp',
        });

        return (
          <AbsoluteFill
            style={{
              background: `linear-gradient(${flowOffset}deg,
                #1a1a2e 0%,
                #16213e 20%,
                #0f3460 40%,
                #e94560 60%,
                #0f3460 80%,
                #1a1a2e 100%
              )`,
              backgroundSize: '400% 400%',
            }}
          />
        );
      case 'epic':
        // Epic 风格背景：用于高潮场景
        const epicAngle = interpolate(frame, [0, 300], [135, 225], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        const epicGlow1 = interpolate(frame, [0, 150, 300], [20, 80, 20], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        const epicGlow2 = interpolate(frame, [0, 150, 300], [80, 20, 80], {
          extrapolateRight: 'clamp',
          easing: Easing.inOut(Easing.ease),
        });
        
        const epicIntensity = interpolate(frame, [0, 60, 180], [0, 0.3, 0.22], {
          extrapolateRight: 'clamp',
        });

        return (
          <AbsoluteFill>
            {/* 深邃但可见的背景 */}
            <AbsoluteFill
              style={{
                background: 'linear-gradient(180deg, #08080e 0%, #0a0a12 50%, #08080e 100%)',
              }}
            />
            
            {/* 动态光晕 1 - 青色 */}
            <div
              style={{
                position: 'absolute',
                top: '30%',
                left: `${epicGlow1}%`,
                width: '60%',
                height: '60%',
                transform: 'translate(-50%, -50%)',
                background: `radial-gradient(ellipse at center, 
                  rgba(103, 232, 249, ${epicIntensity}) 0%, 
                  rgba(56, 189, 248, ${epicIntensity * 0.5}) 30%,
                  transparent 60%
                )`,
                filter: 'blur(80px)',
              }}
            />
            
            {/* 动态光晕 2 - 紫色 */}
            <div
              style={{
                position: 'absolute',
                top: '70%',
                left: `${epicGlow2}%`,
                width: '50%',
                height: '50%',
                transform: 'translate(-50%, -50%)',
                background: `radial-gradient(ellipse at center, 
                  rgba(167, 139, 250, ${epicIntensity * 0.8}) 0%, 
                  rgba(139, 92, 246, ${epicIntensity * 0.4}) 30%,
                  transparent 60%
                )`,
                filter: 'blur(70px)',
              }}
            />
            
            {/* 中心聚焦光 */}
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                width: '40%',
                height: '40%',
                transform: 'translate(-50%, -50%)',
                background: `radial-gradient(ellipse at center, 
                  rgba(255, 255, 255, ${epicIntensity * 0.05}) 0%, 
                  transparent 50%
                )`,
                filter: 'blur(40px)',
              }}
            />
            
            {/* 边缘暗角 - 减弱 */}
            <AbsoluteFill
              style={{
                background: 'radial-gradient(ellipse at center, transparent 45%, rgba(0,0,0,0.3) 100%)',
              }}
            />
          </AbsoluteFill>
        );
      case 'dark':
      default:
        return <AbsoluteFill style={{backgroundColor: color}} />;
    }
  };

  return renderBackground();
};
