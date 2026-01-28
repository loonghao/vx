import React from 'react';
import {
  useCurrentFrame,
  interpolate,
  spring,
  Easing,
} from 'remotion';
import { typography } from '../fonts';

interface TerminalLine {
  type: 'command' | 'output' | 'comment' | 'progress';
  text: string;
  delay?: number; // frames before this line starts
  typeSpeed?: number; // frames per character for typing effect
}

interface TerminalAnimationProps {
  lines: TerminalLine[];
  title?: string;
  delay?: number;
  fontSize?: number;
  showCursor?: boolean;
}

export const TerminalAnimation: React.FC<TerminalAnimationProps> = ({
  lines,
  title = 'Terminal',
  delay = 0,
  fontSize = 16,
  showCursor = true,
}) => {
  const frame = useCurrentFrame();
  const effectiveFrame = Math.max(0, frame - delay);

  // Container animation - more elegant spring
  const containerOpacity = spring({
    frame: effectiveFrame,
    fps: 30,
    config: { damping: 200, stiffness: 80, mass: 0.5 },
  });

  const containerScale = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: { damping: 180, stiffness: 100, mass: 0.6 },
    }),
    [0, 1],
    [0.96, 1]
  );

  const containerY = interpolate(
    spring({
      frame: effectiveFrame,
      fps: 30,
      config: { damping: 200, stiffness: 80, mass: 0.7 },
    }),
    [0, 1],
    [40, 0]
  );

  const glowIntensity = interpolate(effectiveFrame, [0, 90], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.out(Easing.ease),
  });

  // Calculate cumulative delays for each line - FASTER typing
  let cumulativeDelay = 0;
  const lineTimings = lines.map((line, index) => {
    const startFrame = cumulativeDelay + (line.delay || 0);
    // Faster typing: commands at 1.2 fps, others at 0.3
    const typeSpeed = line.typeSpeed || (line.type === 'command' ? 1.2 : 0.3);
    const typingDuration = line.type === 'comment' ? 0 : line.text.length * typeSpeed;
    // Shorter gap between lines (5 frames instead of 8)
    cumulativeDelay = startFrame + typingDuration + 5;
    
    return { startFrame, typeSpeed, typingDuration };
  });

  // Find current cursor line
  const currentLineIndex = lineTimings.findIndex((timing, index) => {
    const nextTiming = lineTimings[index + 1];
    if (!nextTiming) return true;
    return effectiveFrame < nextTiming.startFrame;
  });

  // Group lines by sections (separated by comments)
  const getSectionEndFrame = (lineIndex: number): number => {
    // Find the next comment after this line
    for (let i = lineIndex + 1; i < lines.length; i++) {
      if (lines[i].type === 'comment') {
        return lineTimings[i].startFrame - 10; // Fade out before next section
      }
    }
    return Infinity; // Last section doesn't fade out
  };

  const renderLine = (line: TerminalLine, index: number) => {
    const timing = lineTimings[index];
    const lineFrame = effectiveFrame - timing.startFrame;

    if (lineFrame < 0) return null;

    let displayText = line.text;
    let isTyping = false;

    // Typing effect for commands - faster
    if (line.type === 'command' && lineFrame < timing.typingDuration) {
      const charsToShow = Math.floor(lineFrame / timing.typeSpeed);
      displayText = line.text.substring(0, charsToShow);
      isTyping = true;
    }

    // Progress bar animation - more elegant style
    if (line.type === 'progress') {
      const progressPercent = Math.min(100, Math.floor((lineFrame / timing.typingDuration) * 100));
      const barWidth = 35;
      const filledWidth = Math.floor((progressPercent / 100) * barWidth);
      // Modern progress bar style
      const bar = '▓'.repeat(filledWidth) + '░'.repeat(barWidth - filledWidth);
      displayText = `${bar} ${progressPercent}%`;
    }

    // Fade in animation
    const fadeIn = interpolate(lineFrame, [0, 6], [0, 1], {
      extrapolateRight: 'clamp',
      easing: Easing.out(Easing.ease),
    });

    // Fade out animation for section transitions
    const sectionEndFrame = getSectionEndFrame(index);
    const fadeOut = effectiveFrame > sectionEndFrame 
      ? interpolate(effectiveFrame, [sectionEndFrame, sectionEndFrame + 15], [1, 0], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
        })
      : 1;

    const lineOpacity = fadeIn * fadeOut;

    // Y translation for fade in/out
    const fadeInY = interpolate(lineFrame, [0, 6], [8, 0], {
      extrapolateRight: 'clamp',
      easing: Easing.out(Easing.ease),
    });
    
    const fadeOutY = effectiveFrame > sectionEndFrame
      ? interpolate(effectiveFrame, [sectionEndFrame, sectionEndFrame + 15], [0, -10], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
        })
      : 0;

    return (
      <div
        key={index}
        style={{
          opacity: lineOpacity,
          transform: `translateY(${fadeInY + fadeOutY}px)`,
          display: 'flex',
          alignItems: 'flex-start',
          marginBottom: 6,
        }}
      >
        {line.type === 'command' && (
          <span style={{ color: '#67e8f9', marginRight: 10, fontWeight: 600 }}>❯</span>
        )}
        {line.type === 'output' && (
          <span style={{ color: '#86efac', marginRight: 10, opacity: 0.8 }}>│</span>
        )}
        {line.type === 'progress' && (
          <span style={{ color: '#fbbf24', marginRight: 10, opacity: 0.8 }}>│</span>
        )}
        {line.type === 'comment' && (
          <span style={{ color: '#6b7280', marginRight: 10 }}>#</span>
        )}
        <span
          style={{
            color: getLineColor(line.type),
            fontStyle: line.type === 'comment' ? 'italic' : 'normal',
          }}
          dangerouslySetInnerHTML={{ __html: highlightLine(displayText, line.type) }}
        />
        {/* Cursor - more elegant */}
        {showCursor && isTyping && index === currentLineIndex && (
          <span
            style={{
              backgroundColor: '#67e8f9',
              width: 2,
              height: fontSize + 2,
              display: 'inline-block',
              marginLeft: 1,
              borderRadius: 1,
              opacity: Math.floor(effectiveFrame / 10) % 2 === 0 ? 1 : 0.3,
              boxShadow: '0 0 8px rgba(103, 232, 249, 0.6)',
            }}
          />
        )}
      </div>
    );
  };

  return (
    <div
      style={{
        opacity: containerOpacity,
        transform: `scale(${containerScale}) translateY(${containerY}px)`,
        backgroundColor: 'rgba(10, 10, 15, 0.95)',
        borderRadius: 16,
        overflow: 'hidden',
        border: `1px solid rgba(255, 255, 255, ${0.06 + glowIntensity * 0.04})`,
        boxShadow: `
          0 40px 80px -20px rgba(0, 0, 0, 0.6),
          0 0 ${glowIntensity * 40}px rgba(103, 232, 249, ${glowIntensity * 0.08}),
          inset 0 1px 0 rgba(255, 255, 255, 0.04)
        `,
        width: '100%',
      }}
    >
      {/* Terminal header - more minimal */}
      <div
        style={{
          backgroundColor: 'rgba(20, 20, 28, 0.9)',
          padding: '12px 18px',
          borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', gap: 8 }}>
          <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#ff5f57' }} />
          <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#febc2e' }} />
          <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#28c840' }} />
        </div>
        <span
          style={{
            ...typography.code,
            color: 'rgba(255, 255, 255, 0.35)',
            fontSize: 11,
            fontWeight: 500,
            letterSpacing: '0.1em',
            textTransform: 'uppercase',
          }}
        >
          {title}
        </span>
      </div>

      {/* Terminal content */}
      <div
        style={{
          ...typography.code,
          color: '#e5e7eb',
          fontSize,
          padding: '20px 22px',
          lineHeight: 1.9,
          minHeight: 180,
        }}
      >
        {lines.map((line, index) => renderLine(line, index))}
      </div>
    </div>
  );
};

function getLineColor(type: TerminalLine['type']): string {
  switch (type) {
    case 'command': return '#f1f5f9';
    case 'output': return '#94a3b8';
    case 'comment': return '#64748b';
    case 'progress': return '#fbbf24';
    default: return '#e5e7eb';
  }
}

function highlightLine(text: string, type: TerminalLine['type']): string {
  if (type === 'comment') {
    return escapeHtml(text);
  }

  let result = escapeHtml(text);

  // Highlight vx command (cyan, bold) - make it really stand out
  result = result.replace(/\bvx\b/g, '<span style="color: #67e8f9; font-weight: 700; text-shadow: 0 0 15px rgba(103, 232, 249, 0.5);">vx</span>');

  // Highlight common commands (purple)
  result = result.replace(
    /\b(npx|npm|yarn|pnpm|go|cargo|rustc|uv|uvx|pip|python|java|mvn|curl|wget|bash|brew|apt|winget|choco|node|deno|bun|tar|nvm|irm|iex)\b/g,
    '<span style="color: #c4b5fd;">$1</span>'
  );

  // Highlight flags (muted green)
  result = result.replace(/(\s)(--?[\w-]+)/g, '$1<span style="color: #86efac; opacity: 0.8;">$2</span>');

  // Highlight URLs (blue) - match entire URL including pipes and special chars
  result = result.replace(
    /(https?:\/\/[^\s<>]+)/g,
    '<span style="color: #93c5fd;">$1</span>'
  );

  // Highlight version numbers
  result = result.replace(/@([\d.]+)/g, '@<span style="color: #7dd3fc;">$1</span>');
  result = result.replace(/v([\d.]+)/g, 'v<span style="color: #7dd3fc;">$1</span>');

  // Highlight success messages (green with glow)
  result = result.replace(
    /(✓|Successfully|Installed|Done|Complete|ready)/gi,
    '<span style="color: #86efac; font-weight: 500;">$1</span>'
  );

  // Highlight ⚡ and installation messages
  result = result.replace(
    /(⚡)/g,
    '<span style="color: #fbbf24;">$1</span>'
  );

  result = result.replace(
    /(Installing|Downloading)\s+(\w+)/g,
    '<span style="color: #fbbf24;">$1</span> <span style="color: #67e8f9; font-weight: 600;">$2</span>'
  );

  return result;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
