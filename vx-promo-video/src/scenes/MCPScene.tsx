import React from 'react';
import {AbsoluteFill, useCurrentFrame, spring, interpolate, Easing} from 'remotion';
import {Background} from '../components/Background';
import {FloatingLogos} from '../components/FloatingLogos';
import {typography} from '../fonts';

// Elegant JSON Code Block with syntax highlighting (faster animation)
const JsonCodeBlock: React.FC<{ code: string; delay: number; fontSize: number }> = ({
  code,
  delay,
  fontSize,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  const opacity = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 150, stiffness: 120, mass: 0.4 },
  });

  const scale = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: { damping: 150, stiffness: 150, mass: 0.4 },
    }),
    [0, 1],
    [0.97, 1]
  );

  const translateY = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: { damping: 150, stiffness: 120, mass: 0.5 },
    }),
    [0, 1],
    [25, 0]
  );

  const glowIntensity = interpolate(effectiveFrame, [0, 40], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  const highlightJson = (text: string): string => {
    let result = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Highlight keys (before colon)
    result = result.replace(/"([^"]+)"(\s*:)/g, '<span style="color: #c4b5fd;">"$1"</span>$2');

    // Highlight string values (after colon)
    result = result.replace(/:(\s*)"([^"]+)"/g, ':$1<span style="color: #86efac;">"$2"</span>');

    // Highlight "vx" specifically - make it stand out
    result = result.replace(/"vx"/g, '<span style="color: #67e8f9; font-weight: 700; text-shadow: 0 0 20px rgba(103, 232, 249, 0.5);">"vx"</span>');

    // Highlight brackets and braces
    result = result.replace(/([{}\[\]])/g, '<span style="color: #fcd34d;">$1</span>');

    return result;
  };

  const lines = code.split('\n');

  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale}) translateY(${translateY}px)`,
        backgroundColor: 'rgba(10, 10, 15, 0.95)',
        borderRadius: 20,
        overflow: 'hidden',
        border: `1px solid rgba(167, 139, 250, ${0.15 + glowIntensity * 0.1})`,
        boxShadow: `
          0 40px 80px -20px rgba(0, 0, 0, 0.6),
          0 0 ${glowIntensity * 60}px rgba(167, 139, 250, ${glowIntensity * 0.15}),
          inset 0 1px 0 rgba(255, 255, 255, 0.05)
        `,
        width: 'auto',
      }}
    >
      {/* Header */}
      <div
        style={{
          backgroundColor: 'rgba(20, 20, 30, 0.9)',
          padding: '12px 18px',
          borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', gap: 7 }}>
          <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#ff5f57' }} />
          <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#febc2e' }} />
          <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#28c840' }} />
        </div>
        <span
          style={{
            ...typography.code,
          color: 'rgba(255, 255, 255, 0.6)',
            fontSize: 10,
            fontWeight: 500,
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
          }}
        >
          mcp_config.json
        </span>
      </div>

      {/* Code - Faster line reveal */}
      <pre
        style={{
          ...typography.code,
          color: '#e5e7eb',
          fontSize,
          padding: '20px 24px',
          margin: 0,
          lineHeight: 1.65,
        }}
      >
        {lines.map((line, index) => {
          // Faster staggered fade-in for each line
          const lineDelay = delay + index * 2;
          const lineFrame = Math.max(0, frame - lineDelay);
          const lineOpacity = interpolate(lineFrame, [0, 8], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.out(Easing.ease),
          });
          
          return (
            <div
              key={index}
              style={{ opacity: lineOpacity }}
              dangerouslySetInnerHTML={{ __html: highlightJson(line) || '&nbsp;' }}
            />
          );
        })}
      </pre>
    </div>
  );
};

export const MCPScene: React.FC = () => {
  const frame = useCurrentFrame();

  // Camera: elegant zoom with slight rotation feel
  const cameraZoom = interpolate(frame, [0, 50], [1.1, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.cubic),
  });

  // Subtle downward tilt reveal
  const cameraTiltY = interpolate(frame, [0, 80], [15, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  const titleOpacity = spring({
    frame,
    fps: 30,
    config: { damping: 150, stiffness: 120, mass: 0.4 },
  });

  const titleY = interpolate(
    spring({
      frame,
      fps: 30,
      config: { damping: 150, stiffness: 120, mass: 0.5 },
    }),
    [0, 1],
    [25, 0]
  );

  const mcpCode = `{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@anthropic/mcp-filesystem"]
    },
    "github": {
      "command": "vx",
      "args": ["uvx", "mcp-server-github"]
    },
    "postgres": {
      "command": "vx",
      "args": ["npx", "-y", "@anthropic/mcp-postgres"]
    }
  }
}`;

  const benefits = [
    { icon: '📦', text: '无需预先安装' },
    { icon: '🔄', text: '自动版本管理' },
    { icon: '🌍', text: '任意设备可用' },
  ];

  return (
    <AbsoluteFill>
      <Background variant="apple" />
      
      {/* Floating provider logos - like in space, purple theme */}
      <FloatingLogos 
        count={5} 
        baseSize={50} 
        baseBlur={5} 
        opacity={0.5}
        seed={5}
        logos={['Node.js', 'Python', 'uv', 'npm', 'Terraform']}
      />
      
      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '40px 80px',
          transform: `scale(${cameraZoom}) translateY(${cameraTiltY}px)`,
        }}
      >
        {/* Title - Faster */}
        <div
          style={{
            opacity: titleOpacity,
            transform: `translateY(${titleY}px)`,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            marginBottom: 18,
          }}
        >
          <h1
            style={{
              ...typography.title,
              color: '#ffffff',
              fontSize: 52,
              textAlign: 'center',
              marginBottom: 10,
              background: 'linear-gradient(135deg, #ffffff 0%, #c4b5fd 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
            }}
          >
            AI 原生 MCP 配置
          </h1>
          <p
            style={{
              ...typography.subtitle,
              color: 'rgba(255, 255, 255, 0.6)',
              fontSize: 18,
              textAlign: 'center',
              opacity: interpolate(frame, [8, 18], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }),
            }}
          >
            Claude • Cursor • 任意 MCP 兼容 AI
          </p>
        </div>
        
        {/* JSON Code block - Faster reveal */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            width: '100%',
          }}
        >
          <JsonCodeBlock code={mcpCode} delay={15} fontSize={14} />
        </div>

        {/* Benefits - Faster, earlier */}
        <div
          style={{
            display: 'flex',
            gap: 50,
            marginTop: 35,
          }}
        >
          {benefits.map((item, index) => {
            const itemDelay = 70 + index * 8;
            const itemOpacity = interpolate(
              frame,
              [itemDelay, itemDelay + 12],
              [0, 1],
              { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
            );
            const itemY = interpolate(
              frame,
              [itemDelay, itemDelay + 12],
              [10, 0],
              { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
            );
            return (
              <div
                key={index}
                style={{
                  opacity: itemOpacity,
                  transform: `translateY(${itemY}px)`,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <span style={{ fontSize: 18 }}>{item.icon}</span>
                <span
                  style={{
                    ...typography.body,
                    color: 'rgba(255, 255, 255, 0.7)',
                    fontSize: 14,
                    fontWeight: 500,
                  }}
                >
                  {item.text}
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
