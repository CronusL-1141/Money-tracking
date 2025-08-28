import React from 'react';
import { Box, Typography } from '@mui/material';

interface ScatteredFluxTextProps {
  className?: string;
  size?: number;
}

const ScatteredFluxText: React.FC<ScatteredFluxTextProps> = ({ className = '', size = 24 }) => {
  return (
    <Box 
      className={className}
      sx={{ 
        position: 'relative',
        width: '100%',
        maxWidth: '200px', // 再次增加宽度
        height: '60px',
        userSelect: 'none',
        overflow: 'visible',
        boxSizing: 'border-box',
        margin: '0 auto' // 居中对齐
      }}
    >
      {/* F - 基准线上，轻微左倾 */}
      <Typography 
        component="span"
        sx={{
          position: 'absolute',
          left: '30px', // 再向右移动10px
          top: '8px',
          fontSize: `${size}px`,
          fontWeight: 700,
          background: 'linear-gradient(135deg, #8A9CFF 0%, #9966DD 50%, #FF6EC7 100%)',
          backgroundClip: 'text',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          transform: 'rotate(-5deg)',
          textShadow: '0 2px 6px rgba(138, 156, 255, 0.3)',
          filter: 'brightness(1.2) saturate(1.1)',
          fontFamily: '"Inter", "SF Pro Display", "Segoe UI", sans-serif',
          letterSpacing: '0.1em'
        }}
      >
        F
      </Typography>

      {/* L - 基准线上，向右倾斜 */}
      <Typography 
        component="span"
        sx={{
          position: 'absolute',
          left: '58px', // 再向右移动10px
          top: '10px',
          fontSize: `${size}px`,
          fontWeight: 700,
          background: 'linear-gradient(135deg, #8A9CFF 0%, #9966DD 50%, #FF6EC7 100%)',
          backgroundClip: 'text',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          transform: 'rotate(8deg)',
          textShadow: '0 2px 6px rgba(138, 156, 255, 0.3)',
          filter: 'brightness(1.2) saturate(1.1)',
          fontFamily: '"Inter", "SF Pro Display", "Segoe UI", sans-serif',
          letterSpacing: '0.1em'
        }}
      >
        L
      </Typography>

      {/* U - 基准线上，轻微左倾 */}
      <Typography 
        component="span"
        sx={{
          position: 'absolute',
          left: '90px', // 再向右移动10px
          top: '6px',
          fontSize: `${size}px`,
          fontWeight: 700,
          background: 'linear-gradient(135deg, #8A9CFF 0%, #9966DD 50%, #FF6EC7 100%)',
          backgroundClip: 'text',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          transform: 'rotate(-3deg)',
          textShadow: '0 2px 6px rgba(138, 156, 255, 0.3)',
          filter: 'brightness(1.2) saturate(1.1)',
          fontFamily: '"Inter", "SF Pro Display", "Segoe UI", sans-serif',
          letterSpacing: '0.1em'
        }}
      >
        U
      </Typography>

      {/* X - 基准线上，强烈右倾 */}
      <Typography 
        component="span"
        sx={{
          position: 'absolute',
          left: '125px', // 再向右移动10px
          top: '12px',
          fontSize: `${size}px`,
          fontWeight: 700,
          background: 'linear-gradient(135deg, #8A9CFF 0%, #9966DD 50%, #FF6EC7 100%)',
          backgroundClip: 'text',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          transform: 'rotate(12deg)',
          textShadow: '0 2px 6px rgba(138, 156, 255, 0.3)',
          filter: 'brightness(1.2) saturate(1.1)',
          fontFamily: '"Inter", "SF Pro Display", "Segoe UI", sans-serif',
          letterSpacing: '0.1em'
        }}
      >
        X
      </Typography>

      {/* 添加一些装饰性的光点 */}
      <Box
        sx={{
          position: 'absolute',
          left: '75px',
          top: '15px',
          width: '3px',
          height: '3px',
          backgroundColor: '#4facfe',
          borderRadius: '50%',
          animation: 'sparkle1 2s ease-in-out infinite',
          '@keyframes sparkle1': {
            '0%, 100%': { opacity: 0.3, transform: 'scale(0.8)' },
            '50%': { opacity: 1, transform: 'scale(1.2)' }
          }
        }}
      />
      <Box
        sx={{
          position: 'absolute',
          left: '90px',
          top: '35px',
          width: '2px',
          height: '2px',
          backgroundColor: '#f093fb',
          borderRadius: '50%',
          animation: 'sparkle2 1.8s ease-in-out infinite 0.5s',
          '@keyframes sparkle2': {
            '0%, 100%': { opacity: 0.4, transform: 'scale(1)' },
            '50%': { opacity: 1, transform: 'scale(1.5)' }
          }
        }}
      />
    </Box>
  );
};

export default ScatteredFluxText;