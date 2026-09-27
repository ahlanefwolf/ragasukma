export interface BoneCategory {
  id: string;
  label: string;
  bones: string[];
}

export const BONE_CATEGORIES: BoneCategory[] = [
  { 
    id: 'body', 
    label: 'Badan', 
    bones: ['head', 'chest', 'spine', 'hips'] 
  },
  { 
    id: 'leftArm', 
    label: 'Lengan Kiri', 
    bones: ['leftShoulder', 'leftUpperArm', 'leftLowerArm', 'leftHand'] 
  },
  { 
    id: 'leftFingers', 
    label: 'Jari Kiri', 
    bones: [
      'leftThumb',
      'leftIndex',
      'leftMiddle',
      'leftRing',
      'leftLittle'
    ] 
  },
  { 
    id: 'rightArm', 
    label: 'Lengan Kanan', 
    bones: ['rightShoulder', 'rightUpperArm', 'rightLowerArm', 'rightHand'] 
  },
  { 
    id: 'rightFingers', 
    label: 'Jari Kanan', 
    bones: [
      'rightThumb',
      'rightIndex',
      'rightMiddle',
      'rightRing',
      'rightLittle'
    ] 
  },
  { 
    id: 'legs', 
    label: 'Kaki (L & R)', 
    bones: [
      'leftUpperLeg', 'leftLowerLeg', 'leftFoot',
      'rightUpperLeg', 'rightLowerLeg', 'rightFoot'
    ] 
  },
];

export const BONE_LABELS: Record<string, string> = {
  head: 'Kepala (Head)',
  chest: 'Dada (Chest)',
  spine: 'Punggung (Spine)',
  hips: 'Pinggul (Hips)',

  leftShoulder: 'Bahu Kiri (L-Shoulder)',
  leftUpperArm: 'Lengan Atas Kiri (L-UpperArm)',
  leftLowerArm: 'Lengan Bawah Kiri (L-LowerArm)',
  leftHand: 'Telapak Tangan Kiri (L-Hand)',
  leftThumb: 'Jempol Kiri (L-Thumb)',
  leftIndex: 'Telunjuk Kiri (L-Index)',
  leftMiddle: 'Jari Tengah Kiri (L-Middle)',
  leftRing: 'Jari Manis Kiri (L-Ring)',
  leftLittle: 'Kelingking Kiri (L-Pinky)',

  rightShoulder: 'Bahu Kanan (R-Shoulder)',
  rightUpperArm: 'Lengan Atas Kanan (R-UpperArm)',
  rightLowerArm: 'Lengan Bawah Kanan (R-LowerArm)',
  rightHand: 'Telapak Tangan Kanan (R-Hand)',
  rightThumb: 'Jempol Kanan (R-Thumb)',
  rightIndex: 'Telunjuk Kanan (R-Index)',
  rightMiddle: 'Jari Tengah Kanan (R-Middle)',
  rightRing: 'Jari Manis Kanan (R-Ring)',
  rightLittle: 'Kelingking Kanan (R-Pinky)',

  leftUpperLeg: 'Paha Kiri (L-UpperLeg)',
  leftLowerLeg: 'Betis Kiri (L-LowerLeg)',
  leftFoot: 'Kaki Kiri (L-Foot)',
  rightUpperLeg: 'Paha Kanan (R-UpperLeg)',
  rightLowerLeg: 'Betis Kanan (R-LowerLeg)',
  rightFoot: 'Kaki Kanan (R-Foot)',
};

export const MIRROR_MAP: Record<string, string> = {
  leftShoulder: 'rightShoulder',
  rightShoulder: 'leftShoulder',
  leftUpperArm: 'rightUpperArm',
  rightUpperArm: 'leftUpperArm',
  leftLowerArm: 'rightLowerArm',
  rightLowerArm: 'leftLowerArm',
  leftHand: 'rightHand',
  rightHand: 'leftHand',

  leftThumb: 'rightThumb',
  rightThumb: 'leftThumb',
  leftIndex: 'rightIndex',
  rightIndex: 'leftIndex',
  leftMiddle: 'rightMiddle',
  rightMiddle: 'leftMiddle',
  leftRing: 'rightRing',
  rightRing: 'leftRing',
  leftLittle: 'rightLittle',
  rightLittle: 'leftLittle',

  leftUpperLeg: 'rightUpperLeg',
  rightUpperLeg: 'leftUpperLeg',
  leftLowerLeg: 'rightLowerLeg',
  rightLowerLeg: 'leftLowerLeg',
  leftFoot: 'rightFoot',
  rightFoot: 'leftFoot',
};
