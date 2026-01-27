import React from 'react';
import {
  useCurrentFrame,
  interpolate,
  spring,
  Easing,
} from 'remotion';
import { typography } from '../fonts';

interface CodeBlockProps {
  code: string;
  language?: string;
  delay?: number;
  highlight?: boolean;
  fontSize?: number;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({
  code,
  language = 'terminal',
  delay = 0,
  highlight = true,
  fontSize = 18,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  // Apple style spring animation
  const springConfig = {
    damping: 180,
    stiffness: 120,
    mass: 0.6,
  };

  const opacity = spring({
    frame: effectiveFrame,
    fps: 30,
    config: {
      damping: 200,
      stiffness: 80,
      mass: 0.5,
    },
  });

  const scale = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: springConfig,
    }),
    [0, 1],
    [0.95, 1]
  );

  const translateY = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: {
        damping: 200,
        stiffness: 100,
        mass: 0.7,
      },
    }),
    [0, 1],
    [30, 0]
  );

  const glowIntensity = interpolate(effectiveFrame, [0, 60], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  // Syntax highlighting - process line by line for better control
  const highlightCode = (text: string): string => {
    if (!highlight) return escapeHtml(text);

    const lines = text.split('\n');
    return lines.map(line => {
      // Check if line is a comment first
      if (line.trim().startsWith('#')) {
        return `<span style="color: #6b7280; font-style: italic;">${escapeHtml(line)}</span>`;
      }

      let result = escapeHtml(line);
      
      // Highlight vx command (cyan, bold)
      result = result.replace(/\bvx\b/g, '<span style="color: #22d3ee; font-weight: 600;">vx</span>');
      
      // Highlight common commands (purple)
      result = result.replace(/\b(npx|npm|yarn|pnpm|go|cargo|rustc|uv|uvx|pip|python|java|mvn|curl|bash|brew|apt|winget|choco|node|deno|bun)\b/g, 
        '<span style="color: #a78bfa;">$1</span>');
      
      // Highlight flags (green)
      result = result.replace(/(\s)(--?[\w-]+)/g, '$1<span style="color: #4ade80;">$2</span>');
      
      // Highlight strings in quotes (pink)
      result = result.replace(/"([^"]*)"/g, '<span style="color: #f472b6;">"$1"</span>');
      result = result.replace(/'([^']*)'/g, `<span style="color: #f472b6;">'$1'</span>`);
      
      // Highlight version numbers (blue)
      result = result.replace(/@([\d.]+)/g, '@<span style="color: #38bdf8;">$1</span>');
      
      // Highlight URLs (blue underline)
      result = result.replace(/(https?:\/\/[^\s]+)/g, '<span style="color: #60a5fa; text-decoration: underline;">$1</span>');
      
      // Highlight file paths (yellow)
      result = result.replace(/(\s)([\w./\\-]+\.(toml|json|yaml|yml|js|ts|py|go|rs))/g, 
        '$1<span style="color: #fbbf24;">$2</span>');

      return result;
    }).join('\n');
  };

  const escapeHtml = (text: string): string => {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  };

  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale}) translateY(${translateY}px)`,
        backgroundColor: 'rgba(17, 24, 39, 0.98)',
        borderRadius: 16,
        overflow: 'hidden',
        border: `1px solid rgba(255, 255, 255, ${0.1 + glowIntensity * 0.05})`,
        boxShadow: `
          0 25px 50px -12px rgba(0, 0, 0, 0.5),
          0 0 ${glowIntensity * 30}px rgba(34, 211, 238, ${glowIntensity * 0.1}),
          inset 0 1px 0 rgba(255, 255, 255, 0.05)
        `,
        maxWidth: '90%',
        width: 'auto',
      }}
    >
      {/* Terminal header */}
      <div
        style={{
          backgroundColor: 'rgba(31, 41, 55, 0.95)',
          padding: '12px 16px',
          borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        {/* macOS window buttons */}
        <div style={{ display: 'flex', gap: 8 }}>
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: '50%',
              backgroundColor: '#ef4444',
            }}
          />
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: '50%',
              backgroundColor: '#f59e0b',
            }}
          />
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: '50%',
              backgroundColor: '#22c55e',
            }}
          />
        </div>

        {/* Terminal label */}
        <span
          style={{
            ...typography.body,
          color: 'rgba(255, 255, 255, 0.65)',
            fontSize: 12,
            fontWeight: 500,
            letterSpacing: '0.05em',
            textTransform: 'uppercase',
          }}
        >
          {language}
        </span>
      </div>

      {/* Code content */}
      <pre
        style={{
          ...typography.code,
          color: '#e5e7eb',
          fontSize,
          padding: '24px 28px',
          margin: 0,
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
          lineHeight: 1.7,
        }}
        dangerouslySetInnerHTML={{ __html: highlightCode(code) }}
      />
    </div>
  );
};
