export interface AntigravityModelDef {
  id: string; // e.g. 'ag/gemini-3.8-flash-high'
  name: string;
  category: 'Gemini 3.8' | 'Gemini 3.7' | 'Claude' | 'Gemini Legacy & Pro' | 'Spesialis & Open Weights';
  tag?: string;
  description: string;
}

export const ANTIGRAVITY_PRESET_MODELS: AntigravityModelDef[] = [
  // Gemini 3.8 Series
  {
    id: 'ag/gemini-3.8-flash-high',
    name: 'Gemini 3.8 Flash High',
    category: 'Gemini 3.8',
    tag: 'Rekomendasi',
    description: 'Sangat cepat, reasoning tajam, dan penalaran instan untuk asisten suara.',
  },
  {
    id: 'ag/gemini-3.8-flash-medium',
    name: 'Gemini 3.8 Flash Medium',
    category: 'Gemini 3.8',
    tag: 'Balanced',
    description: 'Keseimbangan optimal antara kecepatan dan kedalaman pemikiran.',
  },
  {
    id: 'ag/gemini-3.8-flash-low',
    name: 'Gemini 3.8 Flash Low',
    category: 'Gemini 3.8',
    tag: 'Low Latency',
    description: 'Latency ultra-rendah untuk interaksi cepat tanpa jeda.',
  },
  {
    id: 'ag/gemini-3.8-pro-high',
    name: 'Gemini 3.8 Pro High',
    category: 'Gemini 3.8',
    tag: 'Pro Reasoning',
    description: 'Penalaran kompleks, analisis logika mendalam, dan coding tingkat lanjut.',
  },

  // Gemini 3.7 Series
  {
    id: 'ag/gemini-3.7-flash-high',
    name: 'Gemini 3.7 Flash High',
    category: 'Gemini 3.7',
    tag: 'Populer',
    description: 'Model Gemini 3.7 dengan performa konsisten, akurat, dan stabil.',
  },
  {
    id: 'ag/gemini-3.7-flash-medium',
    name: 'Gemini 3.7 Flash Medium',
    category: 'Gemini 3.7',
    tag: 'Standard',
    description: 'Model reasoning 3.7 dengan kuota seimbang.',
  },
  {
    id: 'ag/gemini-3.7-flash-low',
    name: 'Gemini 3.7 Flash Low',
    category: 'Gemini 3.7',
    tag: 'Fast',
    description: 'Gemini 3.7 versi hemat kuota dan responsif.',
  },

  // Claude Series
  {
    id: 'ag/claude-sonnet-4-6',
    name: 'Claude Sonnet 4.6',
    category: 'Claude',
    tag: 'Coding & Tools',
    description: 'Kekuatan Claude terbaru untuk coding tingkat tinggi dan tool calling presisi.',
  },
  {
    id: 'ag/claude-opus-4-6-thinking',
    name: 'Claude Opus 4.6 Thinking',
    category: 'Claude',
    tag: 'Deep Thinking',
    description: 'Mode penalaran berpikir mendalam (extended reasoning) untuk masalah rumit.',
  },
  {
    id: 'ag/claude-3-7-sonnet',
    name: 'Claude 3.7 Sonnet',
    category: 'Claude',
    tag: 'Hybrid',
    description: 'Claude 3.7 dengan perpaduan kecepatan dan penalaran mendalam.',
  },
  {
    id: 'ag/claude-3-5-sonnet',
    name: 'Claude 3.5 Sonnet',
    category: 'Claude',
    tag: 'Stable',
    description: 'Claude 3.5 Sonnet standar industri untuk instruksi terstruktur.',
  },

  // Gemini Legacy & Pro
  {
    id: 'ag/gemini-2.5-pro',
    name: 'Gemini 2.5 Pro',
    category: 'Gemini Legacy & Pro',
    tag: 'Multimodal Pro',
    description: 'Model multimodal canggih untuk pemrosesan teks panjang dan konteks besar.',
  },
  {
    id: 'ag/gemini-2.5-flash',
    name: 'Gemini 2.5 Flash',
    category: 'Gemini Legacy & Pro',
    tag: 'Lightweight',
    description: 'Gemini 2.5 Flash ringan dan efisien.',
  },
  {
    id: 'ag/gemini-pro-agent',
    name: 'Gemini Pro Agent',
    category: 'Gemini Legacy & Pro',
    tag: 'Agentic',
    description: 'Model khusus eksekusi tugas dan automasi agen.',
  },
  {
    id: 'ag/gemini-3.6-flash-high',
    name: 'Gemini 3.6 Flash High',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Gemini 3.6 performa tinggi.',
  },
  {
    id: 'ag/gemini-3.6-flash-medium',
    name: 'Gemini 3.6 Flash Medium',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Gemini 3.6 tier medium.',
  },
  {
    id: 'ag/gemini-3.6-flash-low',
    name: 'Gemini 3.6 Flash Low',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Gemini 3.6 tier low.',
  },
  {
    id: 'ag/gemini-3.5-flash-high',
    name: 'Gemini 3.5 Flash High',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Gemini 3.5 generasi stabil.',
  },
  {
    id: 'ag/gemini-3.5-flash-low',
    name: 'Gemini 3.5 Flash Low',
    category: 'Gemini Legacy & Pro',
    tag: '',
    description: 'Gemini 3.5 ultra-ringan.',
  },

  // Spesialis & Open Weights
  {
    id: 'ag/gpt-oss-120b-medium',
    name: 'GPT OSS 120B Medium',
    category: 'Spesialis & Open Weights',
    tag: 'Open Weights',
    description: 'Model open-weight 120 Miliar parameter dengan arsitektur transformer modern.',
  },
  {
    id: 'ag/gemini-3.1-flash-image',
    name: 'Gemini 3.1 Flash Image',
    category: 'Spesialis & Open Weights',
    tag: 'Vision',
    description: 'Spesialis multimodal untuk pemrosesan citra, gambar, dan visual.',
  },
];
