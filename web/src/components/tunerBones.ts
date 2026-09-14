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
      'leftThumbProximal',
      'leftIndexProximal',
      'leftMiddleProximal',
      'leftRingProximal',
      'leftLittleProximal'
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
      'rightThumbProximal',
      'rightIndexProximal',
      'rightMiddleProximal',
      'rightRingProximal',
      'rightLittleProximal'
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
  leftThumbProximal: 'Jempol Kiri (L-Thumb)',
  leftIndexProximal: 'Telunjuk Kiri (L-Index)',
  leftMiddleProximal: 'Jari Tengah Kiri (L-Middle)',
  leftRingProximal: 'Jari Manis Kiri (L-Ring)',
  leftLittleProximal: 'Kelingking Kiri (L-Pinky)',

  rightShoulder: 'Bahu Kanan (R-Shoulder)',
  rightUpperArm: 'Lengan Atas Kanan (R-UpperArm)',
  rightLowerArm: 'Lengan Bawah Kanan (R-LowerArm)',
  rightHand: 'Telapak Tangan Kanan (R-Hand)',
  rightThumbProximal: 'Jempol Kanan (R-Thumb)',
  rightIndexProximal: 'Telunjuk Kanan (R-Index)',
  rightMiddleProximal: 'Jari Tengah Kanan (R-Middle)',
  rightRingProximal: 'Jari Manis Kanan (R-Ring)',
  rightLittleProximal: 'Kelingking Kanan (R-Pinky)',

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

  leftThumbProximal: 'rightThumbProximal',
  rightThumbProximal: 'leftThumbProximal',
  leftIndexProximal: 'rightIndexProximal',
  rightIndexProximal: 'leftIndexProximal',
  leftMiddleProximal: 'rightMiddleProximal',
  rightMiddleProximal: 'leftMiddleProximal',
  leftRingProximal: 'rightRingProximal',
  rightRingProximal: 'leftRingProximal',
  leftLittleProximal: 'rightLittleProximal',
  rightLittleProximal: 'leftLittleProximal',

  leftUpperLeg: 'rightUpperLeg',
  rightUpperLeg: 'leftUpperLeg',
  leftLowerLeg: 'rightLowerLeg',
  rightLowerLeg: 'leftLowerLeg',
  leftFoot: 'rightFoot',
  rightFoot: 'leftFoot',
};
