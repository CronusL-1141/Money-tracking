import React from 'react';
import { Box, Typography } from '@mui/material';

interface FluxLogoProps {
  className?: string;
  size?: number;
  showText?: boolean;
}

const FluxLogo: React.FC<FluxLogoProps> = ({ className = '', size = 40, showText = true }) => {
  return (
    <Box 
      className={className} 
      sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: 1,
        maxWidth: '100%',
        overflow: 'hidden',
        boxSizing: 'border-box'
      }}>
      <svg
        width={size}
        height={size}
        viewBox="0 0 200 200"
        xmlns="http://www.w3.org/2000/svg"
        className="inline-block"
        style={{ 
          maxWidth: '100%',
          maxHeight: '100%',
          overflow: 'visible' // SVG内容可见但容器受限
        }}
      >
        <defs>
          <linearGradient id="fluidGrad1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style={{ stopColor: "#667eea", stopOpacity: 0.9 }} />
            <stop offset="50%" style={{ stopColor: "#764ba2", stopOpacity: 0.8 }} />
            <stop offset="100%" style={{ stopColor: "#f093fb", stopOpacity: 0.9 }} />
          </linearGradient>
          <linearGradient id="fluidGrad2" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style={{ stopColor: "#4facfe", stopOpacity: 0.8 }} />
            <stop offset="100%" style={{ stopColor: "#00f2fe", stopOpacity: 0.9 }} />
          </linearGradient>
          <filter id="blur">
            <feGaussianBlur in="SourceGraphic" stdDeviation="2" />
          </filter>
          <filter id="glow">
            <feGaussianBlur stdDeviation="4" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* 渐变背景 */}
        <rect width="200" height="200" fill="transparent" />

        {/* 主要流体形状 */}
        <path
          d="M50 80 Q100 40 150 80 Q120 120 80 100 Q60 90 50 80 Z"
          fill="url(#fluidGrad1)"
          opacity="0.8"
          filter="url(#glow)"
        >
          <animateTransform
            attributeName="transform"
            type="rotate"
            values="0 100 100;360 100 100"
            dur="20s"
            repeatCount="indefinite"
          />
        </path>

        {/* 辅助流体 */}
        <path
          d="M80 120 Q130 100 160 130 Q140 150 100 140 Q80 135 80 120 Z"
          fill="url(#fluidGrad2)"
          opacity="0.7"
        >
          <animateTransform
            attributeName="transform"
            type="rotate"
            values="360 100 100;0 100 100"
            dur="15s"
            repeatCount="indefinite"
          />
        </path>

        {/* 精致的追踪点 */}
        <g opacity="0.9">
          <circle cx="70" cy="85" r="3" fill="#667eea">
            <animate attributeName="r" values="2;5;2" dur="3s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.5;1;0.5" dur="3s" repeatCount="indefinite" />
          </circle>
          <circle cx="130" cy="95" r="2" fill="#4facfe">
            <animate attributeName="r" values="1;4;1" dur="3s" begin="1s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.6;1;0.6" dur="3s" begin="1s" repeatCount="indefinite" />
          </circle>
          <circle cx="110" cy="125" r="2.5" fill="#f093fb">
            <animate attributeName="r" values="2;4.5;2" dur="3s" begin="2s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.4;1;0.4" dur="3s" begin="2s" repeatCount="indefinite" />
          </circle>
        </g>
      </svg>
      {showText && (
        <Typography 
          variant="h5" 
          component="span" 
          sx={{ 
            fontWeight: 700, 
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%)',
            backgroundClip: 'text',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            letterSpacing: '0.15em',
            textShadow: '0 2px 4px rgba(0,0,0,0.1)',
            fontFamily: '"Inter", "SF Pro Display", "Segoe UI", sans-serif',
            fontSize: size > 32 ? '1.8rem' : '1.3rem'
          }}
        >
FLUX
        </Typography>
      )}
    </Box>
  );
};

export default FluxLogo;