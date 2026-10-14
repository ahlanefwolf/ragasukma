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
      fill="none"
      aria-label="Crystal Fairy Taurus Logo"
    >
      {/* ── 1. WISH GOLD (var(--gi-primary)): Crystal Horns & Crown Diamond ── */}
      <g fill="var(--gi-primary)">
        {/* Left Crystal Horn (Solid, Bold, Perfectly Mirrored) */}
        <polygon points="64,80 168,128 224,192 128,154" />

        {/* Right Crystal Horn (Solid, Bold, Perfectly Mirrored: 512 - x) */}
        <polygon points="448,80 344,128 288,192 384,154" />

        {/* Crown Crystal Primogem Shard */}
        <polygon points="256,120 286,176 256,224 226,176" />
      </g>

      {/* ── 2. WARM WHITE (var(--gi-text-primary)): Crystal Wings & Sprite Body ── */}
      <g fill="var(--gi-text-primary)">
        {/* Left Crystal Wing (Mirrored: 228, 56, 148, 224) */}
        <polygon points="228,230 56,204 148,300 224,272" />

        {/* Right Crystal Wing (Mirrored: 284, 456, 364, 288) */}
        <polygon points="284,230 456,204 364,300 288,272" />

        {/* Central Fairy Crystal Core / Body */}
        <polygon points="256,240 296,328 256,440 216,328" />
      </g>
    </svg>
  );
};
