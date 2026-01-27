import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {FloatingLogos} from '../components/FloatingLogos';
import {ProviderLogos, GenericProviderLogo} from '../components/ProviderLogos';
import {typography} from '../fonts';

// Provider data with SVG logos - reorganized for center focus
const providers = [
  // Inner circle (most important)
  { name: 'Node.js', color: '#68a063', ring: 0, angle: 0 },
  { name: 'Python', color: '#3776ab', ring: 0, angle: 72 },
  { name: 'Go', color: '#00add8', ring: 0, angle: 144 },
  { name: 'Rust', color: '#dea584', ring: 0, angle: 216 },
  { name: 'Java', color: '#f89820', ring: 0, angle: 288 },
  // Middle ring
  { name: 'Deno', color: '#70ffaf', ring: 1, angle: 30 },
  { name: 'Bun', color: '#fbf0df', ring: 1, angle: 90 },
  { name: 'Ruby', color: '#cc342d', ring: 1, angle: 150 },
  { name: 'npm', color: '#cb3837', ring: 1, angle: 210 },
  { name: 'pnpm', color: '#f69220', ring: 1, angle: 270 },
  { name: 'Yarn', color: '#2c8ebb', ring: 1, angle: 330 },
  // Outer ring
  { name: 'uv', color: '#de5fe9', ring: 2, angle: 0 },
  { name: 'Docker', color: '#2496ed', ring: 2, angle: 45 },
  { name: 'Git', color: '#f05032', ring: 2, angle: 90 },
  { name: 'kubectl', color: '#326ce5', ring: 2, angle: 135 },
  { name: 'Terraform', color: '#7b42bc', ring: 2, angle: 180 },
];

const extensions = [
  { name: 'MCP 服务器', icon: '🤖', description: 'AI 工具集成' },
  { name: '项目检测', icon: '🔍', description: '自动识别技术栈' },
  { name: '版本锁定', icon: '🔒', description: '团队一致性' },
  { name: '自定义 Provider', icon: '🧩', description: '按需扩展' },
];

// Orbital logo component
interface OrbitalLogoProps {
  name: string;
  color: string;
  ring: number;
  angle: number;
  index: number;
}

const OrbitalLogo: React.FC<OrbitalLogoProps> = ({
  name,
  color,
  ring,
  angle,
  index,
}) => {
  const frame = useCurrentFrame();
  
  // Ring radii
  const ringRadii = [100, 180, 260];
  const radius = ringRadii[ring];
  
  // Slow orbit rotation
  const orbitSpeed = 0.15 - ring * 0.03;
  const currentAngle = angle + frame * orbitSpeed;
  const angleRad = (currentAngle * Math.PI) / 180;
  
  // Calculate position
  const x = Math.cos(angleRad) * radius;
  const y = Math.sin(angleRad) * radius * 0.5; // Elliptical orbit

  // Entry animation - staggered by ring then angle
  const delay = 20 + ring * 15 + (angle / 360) * 20;
  const effectiveFrame = Math.max(0, frame - delay);

  const entryProgress = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 15, stiffness: 60, mass: 0.6 },
  });

  const opacity = interpolate(effectiveFrame, [0, 20], [0, 1], {
    extrapolateRight: 'clamp',
  });

  // Scale based on Y position (depth effect)
  const depthScale = interpolate(y, [-radius * 0.5, radius * 0.5], [0.7, 1.1]);
  const entryScale = interpolate(entryProgress, [0, 1], [0.3, depthScale]);

  // Glow pulse
  const glowIntensity = interpolate(
    Math.sin(frame * 0.05 + index * 0.8),
    [-1, 1],
    [0.4, 1]
  );

  // Z-index based on Y position
  const zIndex = Math.round(y + 100);

  const LogoComponent = ProviderLogos[name];
  const logoSize = ring === 0 ? 45 : ring === 1 ? 38 : 32;

  return (
    <div
      style={{
        position: 'absolute',
        left: '50%',
        top: '50%',
        transform: `translate(-50%, -50%) translate(${x}px, ${y}px) scale(${entryScale})`,
        opacity,
        zIndex,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 6,
      }}
    >
      {/* Logo container with glow */}
      <div
        style={{
          width: logoSize + 24,
          height: logoSize + 24,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: `
            linear-gradient(
              135deg, 
              rgba(255, 255, 255, 0.06) 0%,
              rgba(255, 255, 255, 0.02) 100%
            )
          `,
          borderRadius: 16,
          border: `1px solid rgba(255, 255, 255, ${0.08 + glowIntensity * 0.05})`,
          boxShadow: `
            0 8px 32px -8px rgba(0, 0, 0, 0.5),
            0 0 ${glowIntensity * 25}px ${color}30,
            inset 0 1px 0 rgba(255, 255, 255, 0.08)
          `,
          backdropFilter: 'blur(8px)',
        }}
      >
        <div style={{ filter: `drop-shadow(0 0 ${glowIntensity * 10}px ${color}60)` }}>
          {LogoComponent ? (
            <LogoComponent size={logoSize} />
          ) : (
            <GenericProviderLogo name={name} color={color} size={logoSize} />
          )}
        </div>
      </div>
      
      {/* Label - only show for inner ring */}
      {ring <= 1 && (
        <span
          style={{
            color: 'rgba(255, 255, 255, 0.8)',
            fontSize: ring === 0 ? 12 : 10,
            fontWeight: 600,
            letterSpacing: '0.02em',
            fontFamily: typography.body.fontFamily,
            textShadow: `0 0 10px ${color}40`,
            whiteSpace: 'nowrap',
          }}
        >
          {name}
        </span>
      )}
    </div>
  );
};

// Orbit rings visualization
const OrbitRings: React.FC = () => {
  const frame = useCurrentFrame();
  
  const opacity = interpolate(frame, [10, 40], [0, 0.15], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const ringRadii = [100, 180, 260];
  const colors = ['#67e8f9', '#a78bfa', '#f472b6'];

  return (
    <svg
      style={{
        position: 'absolute',
        width: 600,
        height: 350,
        left: '50%',
        top: '50%',
        transform: 'translate(-50%, -50%)',
        pointerEvents: 'none',
        opacity,
      }}
    >
      <defs>
        {colors.map((color, i) => (
          <linearGradient key={i} id={`ringGrad${i}`} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor={color} stopOpacity="0.4" />
            <stop offset="50%" stopColor={color} stopOpacity="0.1" />
            <stop offset="100%" stopColor={color} stopOpacity="0.4" />
          </linearGradient>
        ))}
      </defs>
      {ringRadii.map((r, i) => (
        <ellipse
          key={i}
          cx="300"
          cy="175"
          rx={r}
          ry={r * 0.5}
          fill="none"
          stroke={`url(#ringGrad${i})`}
          strokeWidth="1"
          strokeDasharray="8 6"
          strokeDashoffset={frame * (0.3 - i * 0.05)}
        />
      ))}
    </svg>
  );
};

// Extension card with glass effect
interface ExtensionCardProps {
  icon: string;
  name: string;
  description: string;
  delay: number;
  index: number;
}

const ExtensionCard: React.FC<ExtensionCardProps> = ({ icon, name, description, delay, index }) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const scale = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 18, stiffness: 120, mass: 0.4 },
  });

  const opacity = interpolate(effectiveFrame, [0, 12], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const y = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: { damping: 18, stiffness: 100, mass: 0.5 },
    }),
    [0, 1],
    [25, 0]
  );

  const glowColors = ['#67e8f9', '#a78bfa', '#f472b6', '#fbbf24'];
  const glowColor = glowColors[index % 4];

  const glowIntensity = interpolate(
    Math.sin(frame * 0.06 + index * 1.2),
    [-1, 1],
    [0.3, 0.8]
  );

  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale}) translateY(${y}px)`,
        padding: '14px 18px',
        backgroundColor: 'rgba(255, 255, 255, 0.03)',
        borderRadius: 14,
        border: `1px solid rgba(255, 255, 255, ${0.06 + glowIntensity * 0.04})`,
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        boxShadow: `
          0 8px 32px -8px rgba(0, 0, 0, 0.3),
          0 0 ${glowIntensity * 20}px ${glowColor}15,
          inset 0 1px 0 rgba(255, 255, 255, 0.05)
        `,
        backdropFilter: 'blur(10px)',
      }}
    >
      <div
        style={{
          fontSize: 24,
          filter: `drop-shadow(0 0 ${glowIntensity * 8}px ${glowColor}50)`,
        }}
      >
        {icon}
      </div>
      <div>
        <h4
          style={{
            ...typography.title,
            color: '#ffffff',
            fontSize: 13,
            fontWeight: 600,
            marginBottom: 2,
          }}
        >
          {name}
        </h4>
        <p
          style={{
            ...typography.body,
            color: 'rgba(255, 255, 255, 0.6)',
            fontSize: 10,
            margin: 0,
          }}
        >
          {description}
        </p>
      </div>
    </div>
  );
};

export const ProvidersScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera zoom effect - start closer, zoom out
  const cameraZoom = interpolate(frame, [0, 60], [1.15, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  // Title animation
  const titleOpacity = spring({
    frame,
    fps: 30,
    config: { damping: 18, stiffness: 90, mass: 0.5 },
  });

  const titleY = interpolate(
    spring({ frame, fps: 30, config: { damping: 18, stiffness: 100, mass: 0.5 } }),
    [0, 1],
    [30, 0]
  );

  const glowPulse = interpolate(
    Math.sin(frame * 0.04),
    [-1, 1],
    [0.6, 1]
  );

  // Extensions section animation
  const extensionsOpacity = interpolate(frame, [130, 150], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const extensionsY = interpolate(frame, [130, 150], [20, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Background floating logos - very subtle */}
      <FloatingLogos 
        count={4} 
        baseSize={40} 
        baseBlur={8} 
        opacity={0.25}
        seed={7}
      />

      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '30px 60px',
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Title at top */}
        <div
          style={{
            opacity: titleOpacity,
            transform: `translateY(${titleY}px)`,
            textAlign: 'center',
            marginBottom: 20,
            position: 'relative',
            zIndex: 100,
          }}
        >
          <div
            style={{
              position: 'absolute',
              left: '50%',
              top: '50%',
              transform: 'translate(-50%, -50%)',
              width: 350,
              height: 80,
              background: `radial-gradient(ellipse, rgba(103, 232, 249, ${glowPulse * 0.12}) 0%, transparent 70%)`,
              filter: 'blur(20px)',
              pointerEvents: 'none',
            }}
          />
          <h1
            style={{
              ...typography.title,
              fontSize: 48,
              background: 'linear-gradient(135deg, #ffffff 0%, #67e8f9 50%, #a78bfa 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
              marginBottom: 8,
              position: 'relative',
            }}
          >
            丰富的 Provider 生态
          </h1>
          <p
            style={{
              ...typography.subtitle,
              color: 'rgba(255, 255, 255, 0.6)',
              fontSize: 16,
              position: 'relative',
            }}
          >
            20+ 内置 Provider，无限可扩展
          </p>
        </div>

        {/* Central Orbital Logo Display */}
        <div
          style={{
            position: 'relative',
            width: 600,
            height: 350,
            flexShrink: 0,
          }}
        >
          {/* Orbit rings */}
          <OrbitRings />
          
          {/* Center glow */}
          <div
            style={{
              position: 'absolute',
              left: '50%',
              top: '50%',
              transform: 'translate(-50%, -50%)',
              width: 120,
              height: 80,
              background: `radial-gradient(ellipse, rgba(103, 232, 249, ${glowPulse * 0.3}) 0%, rgba(167, 139, 250, ${glowPulse * 0.15}) 50%, transparent 70%)`,
              filter: 'blur(30px)',
              pointerEvents: 'none',
            }}
          />
          
          {/* Orbital logos */}
          {providers.map((provider, index) => (
            <OrbitalLogo
              key={provider.name}
              name={provider.name}
              color={provider.color}
              ring={provider.ring}
              angle={provider.angle}
              index={index}
            />
          ))}
        </div>

        {/* Extensions Section - below logos */}
        <div
          style={{
            opacity: extensionsOpacity,
            transform: `translateY(${extensionsY}px)`,
            width: '100%',
            maxWidth: 900,
            marginTop: 15,
          }}
        >
          <h3
            style={{
              ...typography.body,
              color: 'rgba(255, 255, 255, 0.3)',
              fontSize: 10,
              textTransform: 'uppercase',
              letterSpacing: '0.2em',
              marginBottom: 12,
              textAlign: 'center',
            }}
          >
            扩展系统
          </h3>
          <div style={{ display: 'flex', gap: 14 }}>
            {extensions.map((ext, index) => (
              <ExtensionCard
                key={ext.name}
                icon={ext.icon}
                name={ext.name}
                description={ext.description}
                delay={140 + index * 5}
                index={index}
              />
            ))}
          </div>
        </div>
      </AbsoluteFill>
      
      {/* Vignette effect */}
      <AbsoluteFill
        style={{
          background: 'radial-gradient(ellipse at center, transparent 50%, rgba(0,0,0,0.2) 100%)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
