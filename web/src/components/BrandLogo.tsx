import { Component } from 'solid-js';

export const BrandLogo: Component<{ size?: number; class?: string }> = (props) => {
  const s = props.size || 36;
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 512 512"
      width={s}
      height={s}
      class={props.class || ''}
    >
      {/* Base Squircle */}
      <rect x="16" y="16" width="480" height="480" rx="116" fill="#FAF7F2" />

      {/* Headset Headband Arc */}
      <path
        d="M 86 250 A 170 170 0 0 1 426 250"
        fill="none"
        stroke="#29483A"
        stroke-width="28"
        stroke-linecap="round"
      />

      {/* Earcups */}
      <rect x="56" y="192" width="60" height="116" rx="30" fill="#596B45" />
      <rect x="396" y="192" width="60" height="116" rx="30" fill="#596B45" />

      {/* Boom Microphone */}
      <path
        d="M 86 280 C 86 375, 145 395, 204 375"
        fill="none"
        stroke="#29483A"
        stroke-width="14"
        stroke-linecap="round"
      />
      <rect x="208" y="362" width="28" height="22" rx="11" fill="#B08A4A" />

      {/* Smiling Eyes */}
      <path
        d="M 170 236 Q 194 204, 218 236"
        fill="none"
        stroke="#29483A"
        stroke-width="16"
        stroke-linecap="round"
      />
      <path
        d="M 294 236 Q 318 204, 342 236"
        fill="none"
        stroke="#29483A"
        stroke-width="16"
        stroke-linecap="round"
      />

      {/* Blush Pills */}
      <rect x="144" y="262" width="42" height="16" rx="8" fill="#B08A4A" />
      <rect x="326" y="262" width="42" height="16" rx="8" fill="#B08A4A" />

      {/* Voice Soundwave Mouth */}
      <rect x="226" y="290" width="12" height="34" rx="6" fill="#B08A4A" />
      <rect x="250" y="278" width="12" height="58" rx="6" fill="#596B45" />
      <rect x="274" y="290" width="12" height="34" rx="6" fill="#B08A4A" />
    </svg>
  );
};
