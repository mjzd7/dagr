import React from 'react';

interface BrandLogoProps {
  size?: number;
  className?: string;
  inverted?: boolean;
}

export function BrandLogo({ size = 20, className = '', inverted = false }: BrandLogoProps) {
  const strokeColor = inverted ? '#FFFFFF' : '#000000';
  const nodeFill = inverted ? '#FFFFFF' : '#000000';
  const apexFill = inverted ? '#000000' : '#FFFFFF';
  const apexStroke = inverted ? '#FFFFFF' : '#000000';

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <line x1="3" y1="3" x2="8" y2="12" stroke={strokeColor} strokeWidth="2.5" strokeLinecap="round" />
      <line x1="13" y1="3" x2="8" y2="12" stroke={strokeColor} strokeWidth="2.5" strokeLinecap="round" />
      <circle cx="3" cy="3" r="2" fill={nodeFill} />
      <circle cx="13" cy="3" r="2" fill={nodeFill} />
      <circle cx="8" cy="12" r="2.5" fill={apexFill} stroke={apexStroke} strokeWidth="1.5" />
    </svg>
  );
}

export function BrandLogoBadge({ size = 32, className = '' }: { size?: number; className?: string }) {
  return (
    <div
      style={{ width: size, height: size }}
      className={`rounded-xl bg-white text-black flex items-center justify-center p-1.5 shadow-lg group-hover:scale-105 transition-transform shrink-0 ${className}`}
    >
      <BrandLogo size={Math.round(size * 0.625)} />
    </div>
  );
}
