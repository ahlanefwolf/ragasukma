import { Component } from 'solid-js';
import { MobileApp } from '../mobile/MobileApp';

interface MobileClientViewProps {
  authKey: string;
  showToast: (msg: string) => void;
}

export const MobileClientView: Component<MobileClientViewProps> = (props) => {
  return (
    <div class="fixed inset-0 w-full h-full overflow-hidden touch-none select-none">
      <MobileApp authKey={props.authKey} showToast={props.showToast} />
    </div>
  );
};
