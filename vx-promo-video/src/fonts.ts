import { loadFont as loadInter } from "@remotion/google-fonts/Inter";
import { loadFont as loadJetBrainsMono } from "@remotion/google-fonts/JetBrainsMono";
import { staticFile, continueRender, delayRender } from "remotion";

// Load JetBrains Mono for code blocks - developer-friendly monospace
const { fontFamily: jetbrainsFamily } = loadJetBrainsMono();

// TencentSans Bold font family name (after loading the font)
const tencentSansFamily = "TencentSans Bold";

// Load local TencentSans Bold font
const fontFace = new FontFace(
  tencentSansFamily,
  `url('${staticFile("fonts/TencentSansBold.ttf")}') format('truetype')`
);

// Load font asynchronously
fontFace.load().then((loadedFace) => {
  document.fonts.add(loadedFace);
}).catch((error) => {
  console.warn("Failed to load TencentSans Bold font:", error);
});

// Font configurations - Using TencentSans W7 for Chinese display
export const fonts = {
  // Primary display font for titles (TencentSans W7 for Chinese)
  display: `"${tencentSansFamily}", "PingFang SC", "Microsoft YaHei", "Noto Sans SC", -apple-system, sans-serif`,
  
  // Secondary text font for body/subtitles (same as display for consistency)
  text: `"${tencentSansFamily}", "PingFang SC", "Microsoft YaHei", "Noto Sans SC", -apple-system, sans-serif`,
  
  // Monospace font for code blocks
  mono: `${jetbrainsFamily}, "SF Mono", "Fira Code", "Cascadia Code", Menlo, Monaco, Consolas, monospace`,
};

// Typography presets
export const typography = {
  title: {
    fontFamily: fonts.display,
    fontWeight: 700,
    letterSpacing: '0.02em', // Slightly wider for Chinese
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
  },
  subtitle: {
    fontFamily: fonts.text,
    fontWeight: 400,
    letterSpacing: '0.01em',
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
  },
  body: {
    fontFamily: fonts.text,
    fontWeight: 400,
    letterSpacing: '0.01em',
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
  },
  code: {
    fontFamily: fonts.mono,
    fontWeight: 400,
    letterSpacing: '0',
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
  },
  button: {
    fontFamily: fonts.display,
    fontWeight: 600,
    letterSpacing: '0.02em',
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
  },
};
