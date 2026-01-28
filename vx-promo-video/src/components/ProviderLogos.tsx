import React from 'react';

// Provider logo components using official colors and simple SVG representations
export const ProviderLogos: Record<string, React.FC<{size?: number}>> = {
  // Node.js - Official green hexagon
  'Node.js': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 289" fill="none">
      <path d="M128 0L0 73.9v144.4l128 73.9 128-73.9V73.9L128 0z" fill="#8CC84B"/>
      <path d="M128 288.1l-128-73.9v-72.9l128 73.9 128-73.9v72.9l-128 73.9z" fill="#6BA53B"/>
      <path d="M128 215.2l128-73.9V68.4L128 142.3 0 68.4v72.9l128 73.9z" fill="#8CC84B"/>
    </svg>
  ),
  
  // Python - Blue and yellow
  'Python': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 255" fill="none">
      <defs>
        <linearGradient id="pythonGrad1" x1="12.96%" x2="79.64%" y1="12.04%" y2="78.01%">
          <stop offset="0%" stopColor="#387EB8"/>
          <stop offset="100%" stopColor="#366994"/>
        </linearGradient>
        <linearGradient id="pythonGrad2" x1="19.13%" x2="90.58%" y1="20.58%" y2="88.34%">
          <stop offset="0%" stopColor="#FFE052"/>
          <stop offset="100%" stopColor="#FFC331"/>
        </linearGradient>
      </defs>
      <path fill="url(#pythonGrad1)" d="M126.9.138c-64.8 0-60.8 28.1-60.8 28.1l.1 29.2h61.9v8.7H39.1S0 61.538 0 126.938c0 65.4 34.1 63.1 34.1 63.1h20.3v-30.4s-1.1-34.1 33.5-34.1h57.7s32.5.5 32.5-31.4V33.138s4.9-33-51.2-33z"/>
      <path fill="url(#pythonGrad2)" d="M128.8 254.138c64.8 0 60.8-28.1 60.8-28.1l-.1-29.2h-61.9v-8.7h89.1s39.1 4.5 39.1-60.9c0-65.4-34.1-63.1-34.1-63.1h-20.3v30.4s1.1 34.1-33.5 34.1h-57.7s-32.5-.5-32.5 31.4v56.9s-4.9 33 51.1 33z"/>
      <ellipse cx="94.6" cy="45.2" fill="#FFF" rx="12" ry="12.3"/>
      <ellipse cx="161.2" cy="209" fill="#FFF" rx="12" ry="12.3"/>
    </svg>
  ),
  
  // Go - Gopher blue
  'Go': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 348" fill="none">
      <path fill="#00ADD8" d="M1.3 173.8c0-1 .6-1.5 1.7-1.5h7.4c1 0 1.6.5 1.6 1.4 0 2 1.5 3 4.4 3h27.3c4.1 0 7.2-2.2 7.2-5.6 0-2-.7-3.6-2.3-4.8-1.5-1.2-4.3-2.4-8.3-3.4L24.4 159c-7.2-1.9-12.4-4.7-15.5-8.5-3.1-3.8-4.7-8.8-4.7-15 0-7.3 2.6-13.2 7.7-17.6 5.2-4.5 11.7-6.7 19.5-6.7h31.2c6 0 10.7 1.6 14.2 4.7 3.5 3.2 5.2 7.3 5.2 12.5 0 1-.5 1.5-1.6 1.5h-7.5c-1 0-1.6-.5-1.6-1.4 0-2.6-1.8-3.9-5.3-3.9H33.4c-4.7 0-8.2 2.3-8.2 6.5 0 2.3.8 4.1 2.5 5.3 1.7 1.2 4.5 2.3 8.5 3.3l16 3.6c7.2 1.7 12.4 4.3 15.4 7.9 3 3.6 4.5 8.4 4.5 14.4 0 7.6-2.6 13.6-7.8 18-5.2 4.5-12 6.7-20.4 6.7H14.2c-5.6 0-10.1-1.6-13.5-4.9-3.4-3.2-5.1-7.6-5.1-13.1 0-1 .6-1.5 1.7-1.5h7.4"/>
      <path fill="#00ADD8" d="M128.3 193.5c-11.8 0-21.3-3.8-28.6-11.3-7.3-7.5-10.9-17.3-10.9-29.3 0-12.1 3.7-21.9 11-29.5 7.4-7.6 17-11.3 28.8-11.3 11.8 0 21.3 3.8 28.5 11.3 7.2 7.5 10.9 17.4 10.9 29.5 0 12-3.6 21.8-10.9 29.3-7.3 7.5-16.9 11.3-28.8 11.3z"/>
      <path fill="#FFF" d="M128.3 179.3c7.4 0 13.2-2.4 17.4-7.3 4.2-4.8 6.3-11.3 6.3-19.3 0-8.1-2.1-14.6-6.3-19.5-4.2-4.9-9.9-7.3-17.1-7.3-7.3 0-13.1 2.4-17.3 7.3-4.2 4.8-6.3 11.3-6.3 19.4 0 8 2.1 14.5 6.2 19.4 4.1 4.8 9.8 7.3 17.1 7.3z"/>
    </svg>
  ),
  
  // Rust - Orange/brown gear
  'Rust': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <circle cx="128" cy="128" r="120" fill="#000" stroke="#F74C00" strokeWidth="16"/>
      <text x="128" y="150" textAnchor="middle" fill="#F74C00" fontSize="100" fontWeight="bold" fontFamily="serif">R</text>
    </svg>
  ),
  
  // Java - Red/blue coffee cup (simplified)
  'Java': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 346" fill="none">
      <path fill="#5382A1" d="M82.6 267.4s-13.5 7.9 9.6 10.5c28 3.2 42.2 2.7 73-3 0 0 8.1 5.1 19.4 9.5-69 29.6-156.3-1.7-102-17z"/>
      <path fill="#5382A1" d="M74.2 229.6s-15.1 11.2 8 13.5c29.9 3 53.4 3.2 94.2-4.4 0 0 5.6 5.7 14.5 8.8-83.5 24.4-176.5 1.9-116.7-17.9z"/>
      <path fill="#E76F00" d="M143.9 165.5c17 19.5-4.5 37.1-4.5 37.1s43.1-22.3 23.3-50.2c-18.5-26-32.7-38.9 44.1-83.4 0 0-120.5 30.1-62.9 96.5z"/>
      <path fill="#5382A1" d="M233.4 295.3s10 8.2-11 14.6c-39.8 12.1-165.8 15.7-200.8.5-12.6-5.5 11-13 18.4-14.6 7.7-1.7 12.1-1.4 12.1-1.4-13.9-9.8-90 19.2-38.6 27.5 140.1 22.6 255.5-10.2 219.9-26.6z"/>
    </svg>
  ),
  
  // Deno - Simple dinosaur
  'Deno': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 512 512" fill="none">
      <circle cx="256" cy="256" r="230" fill="#222"/>
      <ellipse cx="256" cy="256" rx="200" ry="200" fill="#222"/>
      <path fill="#FFF" d="M256 90c91.8 0 166 74.2 166 166s-74.2 166-166 166S90 347.8 90 256 164.2 90 256 90z"/>
      <circle cx="320" cy="200" r="30" fill="#222"/>
    </svg>
  ),
  
  // Bun - Bun logo (cream/brown)
  'Bun': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <circle cx="128" cy="128" r="120" fill="#FBF0DF"/>
      <ellipse cx="90" cy="130" rx="15" ry="20" fill="#222"/>
      <ellipse cx="166" cy="130" rx="15" ry="20" fill="#222"/>
      <path fill="#F88379" d="M128 180c-30 0-50-15-50-30h100c0 15-20 30-50 30z"/>
    </svg>
  ),
  
  // Ruby - Red gem
  'Ruby': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 255" fill="none">
      <polygon fill="#CC342D" points="128,0 0,170 128,255 256,170"/>
      <polygon fill="#9A2820" points="128,100 0,170 128,255 256,170"/>
      <polygon fill="#CC342D" points="128,0 60,85 128,100 196,85"/>
    </svg>
  ),
  
  // npm - Red box
  'npm': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <rect width="256" height="256" fill="#CB3837"/>
      <path fill="#FFF" d="M48 48v160h80v-128h48v128h32V48z"/>
    </svg>
  ),
  
  // pnpm - Yellow boxes
  'pnpm': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <rect x="16" y="16" width="70" height="70" fill="#F69220"/>
      <rect x="93" y="16" width="70" height="70" fill="#F69220"/>
      <rect x="170" y="16" width="70" height="70" fill="#F69220"/>
      <rect x="16" y="93" width="70" height="70" fill="#F69220"/>
      <rect x="93" y="93" width="70" height="70" fill="#4E4E4E"/>
      <rect x="16" y="170" width="70" height="70" fill="#4E4E4E"/>
      <rect x="93" y="170" width="70" height="70" fill="#F69220"/>
    </svg>
  ),
  
  // Yarn - Blue cat
  'Yarn': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <circle cx="128" cy="128" r="120" fill="#2C8EBB"/>
      <path fill="#FFF" d="M180 80c-20-30-60-20-70 10-5 15 0 30 10 40 10 10 10 30-10 40-15 10-40 10-50-5-15 20-5 45 20 55 30 10 60-5 70-30 10 10 30 5 35-10 5-20-15-35-35-30 15-20 45-30 30-70z"/>
    </svg>
  ),
  
  // uv - Purple lightning
  'uv': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <circle cx="128" cy="128" r="120" fill="#DE5FE9"/>
      <path fill="#FFF" d="M140 50l-60 90h45l-15 66 60-90h-45z"/>
    </svg>
  ),
  
  // Docker - Blue whale
  'Docker': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 185" fill="none">
      <path fill="#2496ED" d="M250.7 78.8c-3.5-2.4-11.5-3.3-17.6-2.1-1.8-13-9.2-24.3-22.6-34.5l-7.7-5.1-5.1 7.7c-6.5 9.8-8.2 25.8-1.3 36.4-3.1 1.7-9 3.9-16.9 3.8H2.2l-.6 3c-2.1 12.5-2.1 51.5 29.3 81.4C52.8 191.1 82.7 202 122.8 202c84.2 0 146.5-38.7 175.8-109 11.5.2 36.2-.1 48.9-24.2l3.2-5.8-7.3-4.8c-2.6-1.7-8.6-4.7-15.8-4.3-7.1.4-13.6 2.7-19 8.2 0 0 3.2-6.2.1-8.3z"/>
      <path fill="#FFF" d="M49 92H28v21h21V92zM78 92H57v21h21V92zM107 92H86v21h21V92zM136 92h-21v21h21V92zM78 64H57v21h21V64zM107 64H86v21h21V64zM136 64h-21v21h21V64zM165 64h-21v21h21V64zM107 36H86v21h21V36z"/>
    </svg>
  ),
  
  // Git - Orange F
  'Git': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 256" fill="none">
      <path fill="#F05032" d="M251.2 118.5L137.5 4.8c-6.4-6.4-16.8-6.4-23.2 0l-23.5 23.5 29.8 29.8c6.9-2.3 14.8-.7 20.3 4.8 5.5 5.5 7.1 13.5 4.7 20.4l28.7 28.7c6.9-2.3 14.9-.8 20.4 4.8 7.7 7.7 7.7 20.2 0 27.9-7.7 7.7-20.2 7.7-27.9 0-5.8-5.8-7.2-14.4-4.2-21.5l-26.8-26.8v70.5c1.9.9 3.6 2.1 5.2 3.6 7.7 7.7 7.7 20.2 0 27.9-7.7 7.7-20.2 7.7-27.9 0-7.7-7.7-7.7-20.2 0-27.9 2-2 4.3-3.4 6.8-4.3V118c-2.5-.9-4.8-2.3-6.8-4.3-5.9-5.9-7.2-14.6-4.1-21.7L80 63.1l-75.2 75.2c-6.4 6.4-6.4 16.8 0 23.2L118.5 275c6.4 6.4 16.8 6.4 23.2 0L251.2 165.5c6.4-6.4 6.4-16.8 0-23.2z"/>
    </svg>
  ),
  
  // kubectl - Kubernetes blue wheel
  'kubectl': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 249" fill="none">
      <path fill="#326CE5" d="M128.8 0L0 73.8v100l128.8 74.5 128.8-74.5v-100L128.8 0z"/>
      <path fill="#FFF" d="M128.8 50l-60 35v70l60 35 60-35v-70l-60-35z"/>
      <circle cx="128.8" cy="120" r="25" fill="#326CE5"/>
    </svg>
  ),
  
  // Terraform - Purple T
  'Terraform': ({size = 40}) => (
    <svg width={size} height={size} viewBox="0 0 256 289" fill="none">
      <polygon fill="#5C4EE5" points="89.7,0 89.7,102.5 176.2,51.3 176.2,0"/>
      <polygon fill="#4040B2" points="0,51.3 0,153.8 86.5,102.5 86.5,0"/>
      <polygon fill="#5C4EE5" points="89.7,110.7 89.7,213.3 176.2,162 176.2,59.5"/>
      <polygon fill="#5C4EE5" points="179.4,162 179.4,264.5 256,218.8 256,116.3"/>
    </svg>
  ),
};

// Generic fallback for unknown providers
export const GenericProviderLogo: React.FC<{name: string; color: string; size?: number}> = ({
  name,
  color,
  size = 40,
}) => (
  <svg width={size} height={size} viewBox="0 0 100 100" fill="none">
    <circle cx="50" cy="50" r="45" fill={color} opacity="0.2"/>
    <circle cx="50" cy="50" r="35" fill={color} opacity="0.4"/>
    <text 
      x="50" 
      y="58" 
      textAnchor="middle" 
      fill={color}
      fontSize="32"
      fontWeight="bold"
      fontFamily="system-ui, sans-serif"
    >
      {name.charAt(0).toUpperCase()}
    </text>
  </svg>
);
