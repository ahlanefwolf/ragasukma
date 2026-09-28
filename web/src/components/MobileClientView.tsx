import { Component } from 'solid-js';
import { MobileApp } from '../mobile/MobileApp';

interface MobileClientViewProps {
  authKey: string;
  showToast: (msg: string) => void;
}

export const MobileClientView: Component<MobileClientViewProps> = (props) => {
  return (
    <div class="h-full w-full overflow-hidden">
      <MobileApp authKey={props.authKey} showToast={props.showToast} />
    </div>
  );
};
