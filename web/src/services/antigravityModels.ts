export interface AntigravityModelDef {
  id: string; // e.g. 'ag/gemini-3.8-flash-high'
  name: string;
  category: 'Gemini 3.8' | 'Gemini 3.7' | 'Claude' | 'Gemini Legacy & Pro' | 'Specialist & Open Weights';
  tag?: string;
  description: string;
}

export const ANTIGRAVITY_PRESET_MODELS: AntigravityModelDef[] = [
  // Gemini 3.8 Series
  {
    id: 'ag/gemini-3.8-flash-high',
    name: 'Gemini 3.8 Flash High',
    category: 'Gemini 3.8',
    tag: 'Recommended',
    description: 'Extremely fast, sharp reasoning, and instant thought latency for voice assistants.',
  },
  {
    id: 'ag/gemini-3.8-flash-medium',
    name: 'Gemini 3.8 Flash Medium',
    category: 'Gemini 3.8',
    tag: 'Balanced',
    description: 'Optimal balance between speed and reasoning depth.',
  },
  {
    id: 'ag/gemini-3.8-flash-low',
    name: 'Gemini 3.8 Flash Low',
    category: 'Gemini 3.8',
    tag: 'Low Latency',
    description: 'Ultra-low latency for quick and uninterrupted interaction.',
  },
  {
    id: 'ag/gemini-3.8-pro-high',
    name: 'Gemini 3.8 Pro High',
    category: 'Gemini 3.8',
    tag: 'Pro Reasoning',
    description: 'Complex reasoning, deep logical deductions, and advanced coding capability.',
  },

  // Gemini 3.7 Series
  {
    id: 'ag/gemini-3.7-flash-high',
    name: 'Gemini 3.7 Flash High',
    category: 'Gemini 3.7',
    tag: 'Popular',
    description: 'Gemini 3.7 model offering consistent, accurate, and stable output.',
  },
  {
    id: 'ag/gemini-3.7-flash-medium',
    name: 'Gemini 3.7 Flash Medium',
    category: 'Gemini 3.7',
    tag: 'Standard',
    description: '3.7 reasoning model with balanced token quota usage.',
  },
  {
    id: 'ag/gemini-3.7-flash-low',
    name: 'Gemini 3.7 Flash Low',
    category: 'Gemini 3.7',
    tag: 'Fast',
    description: 'Quota-friendly and highly responsive Gemini 3.7 tier.',
  },

  // Claude Series
  {
    id: 'ag/claude-sonnet-4-6',
    name: 'Claude Sonnet 4.6',
    category: 'Claude',
    tag: 'Coding & Tools',
    description: 'Latest Claude strength for high-level coding and precise tool calling.',
  },
  {
    id: 'ag/claude-opus-4-6-thinking',
    name: 'Claude Opus 4.6 Thinking',
    category: 'Claude',
    tag: 'Deep Thinking',
    description: 'Extended reasoning mode for complex and intricate problem-solving.',
  },
  {
    id: 'ag/claude-3-7-sonnet',
    name: 'Claude 3.7 Sonnet',
    category: 'Claude',
    tag: 'Hybrid',
    description: 'Claude 3.7 combining high throughput speed with deep reasoning.',
  },
  {
    id: 'ag/claude-3-5-sonnet',
    name: 'Claude 3.5 Sonnet',
    category: 'Claude',
    tag: 'Stable',
    description: 'Industry-standard Claude 3.5 Sonnet for structured instructions and formatting.',
  },

  // Gemini Legacy & Pro
  {
    id: 'ag/gemini-2.5-pro',
    name: 'Gemini 2.5 Pro',
    category: 'Gemini Legacy & Pro',
    tag: 'Multimodal Pro',
    description: 'Advanced multimodal model for lengthy inputs and large context windows.',
  },
  {
    id: 'ag/gemini-2.5-flash',
    name: 'Gemini 2.5 Flash',
    category: 'Gemini Legacy & Pro',
    tag: 'Lightweight',
    description: 'Lightweight, fast, and resource-efficient Gemini 2.5.',
  },
  {
    id: 'ag/gemini-pro-agent',
    name: 'Gemini Pro Agent',
    category: 'Gemini Legacy & Pro',
    tag: 'Agentic',
    description: 'Dedicated model optimized for task execution and agentic tool automation.',
  },
  {
    id: 'ag/gemini-3.6-flash-high',
    name: 'Gemini 3.6 Flash High',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'High-performance Gemini 3.6.',
  },
  {
    id: 'ag/gemini-3.6-flash-medium',
    name: 'Gemini 3.6 Flash Medium',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Medium tier Gemini 3.6.',
  },
  {
    id: 'ag/gemini-3.6-flash-low',
    name: 'Gemini 3.6 Flash Low',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Low latency tier Gemini 3.6.',
  },
  {
    id: 'ag/gemini-3.5-flash-high',
    name: 'Gemini 3.5 Flash High',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Stable generation Gemini 3.5.',
  },
  {
    id: 'ag/gemini-3.5-flash-low',
    name: 'Gemini 3.5 Flash Low',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Ultra-lightweight Gemini 3.5.',
  },

  // Specialist & Open Weights
  {
    id: 'ag/gpt-oss-120b-medium',
    name: 'GPT OSS 120B Medium',
    category: 'Specialist & Open Weights',
    tag: 'Open Weights',
    description: '120 Billion parameter open-weights model with modern transformer architecture.',
  },
  {
    id: 'ag/gemini-3.1-flash-image',
    name: 'Gemini 3.1 Flash Image',
    category: 'Specialist & Open Weights',
    tag: 'Vision',
    description: 'Multimodal specialist tailored for visual processing and image understanding.',
  },
];
