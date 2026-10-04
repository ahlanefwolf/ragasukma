export interface BoneCategory {
  id: string;
  label: string;
  bones: string[];
}

export const BONE_CATEGORIES: BoneCategory[] = [
  { 
    id: 'body', 
    label: 'Body', 
    bones: ['head', 'chest', 'spine', 'hips'] 
  },
  { 
    id: 'leftArm', 
    label: 'Left Arm', 
    bones: ['leftShoulder', 'leftUpperArm', 'leftLowerArm', 'leftHand'] 
  },
  { 
    id: 'leftFingers', 
    label: 'Left Fingers', 
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
    label: 'Right Arm', 
    bones: ['rightShoulder', 'rightUpperArm', 'rightLowerArm', 'rightHand'] 
  },
  { 
    id: 'rightFingers', 
    label: 'Right Fingers', 
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
    label: 'Legs (L & R)', 
    bones: [
      'leftUpperLeg', 'leftLowerLeg', 'leftFoot',
      'rightUpperLeg', 'rightLowerLeg', 'rightFoot'
    ] 
  },
];

export const BONE_LABELS: Record<string, string> = {
  head: 'Head',
  chest: 'Chest',
  spine: 'Spine',
  hips: 'Hips',

  leftShoulder: 'Left Shoulder',
  leftUpperArm: 'Left Upper Arm',
  leftLowerArm: 'Left Lower Arm',
  leftHand: 'Left Hand',
  leftThumb: 'Left Thumb',
  leftIndex: 'Left Index',
  leftMiddle: 'Left Middle',
  leftRing: 'Left Ring',
  leftLittle: 'Left Pinky',

  rightShoulder: 'Right Shoulder',
  rightUpperArm: 'Right Upper Arm',
  rightLowerArm: 'Right Lower Arm',
  rightHand: 'Right Hand',
  rightThumb: 'Right Thumb',
  rightIndex: 'Right Index',
  rightMiddle: 'Right Middle',
  rightRing: 'Right Ring',
  rightLittle: 'Right Pinky',

  leftUpperLeg: 'Left Upper Leg',
  leftLowerLeg: 'Left Lower Leg',
  leftFoot: 'Left Foot',
  rightUpperLeg: 'Right Upper Leg',
  rightLowerLeg: 'Right Lower Leg',
  rightFoot: 'Right Foot',
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
