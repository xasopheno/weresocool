import { createContext } from 'react';
import { IMarker } from 'react-ace';
import { language_template } from '../app/components/Editor/language_template';
import { ResponseType } from '../app/actions/actions';

interface Editor {
  [index: number]: { name: string; style: string };
}

export type Fetch =
  | { state: 'loading' }
  | { state: 'good' }
  | { state: 'bad'; error: Error; unauthorized?: true };

export const Editors: Editor = {
  0: { name: 'Text', style: '' },
  1: { name: 'Vim', style: 'vim' },
  2: { name: 'Emacs', style: 'emacs' },
};

export interface Store {
  editor: number;
  backend: Fetch;
  render: ResponseType;
  language: string;
  errorMessage: string;
  markers: IMarker[];
  initializeTest: boolean;
}

export const intialStore: Store = {
  editor: 1,
  backend: { state: 'bad', error: Error('Startup') },
  render: ResponseType.RenderSuccess,
  language: language_template,
  errorMessage: '',
  markers: [],
  initializeTest: true,
};

export const testStore: Store = {
  editor: 1,
  backend: { state: 'good' },
  render: ResponseType.RenderSuccess,
  language: language_template,
  errorMessage: '',
  markers: [],
  initializeTest: false,
};
export const GlobalContext = createContext((undefined as unknown) as Store);
// // This adds a `_k: 'something'` to a given type
// type K<TKey extends string, TValue> = {_k: TKey} & TValue;
// export type Action =
// | K<'InitialAuth', InitialAuth>
// | K<'LoginSuccess', LoginSuccess>
// | K<'ClickedLogout', {}>
// | K<'ClickedLogout/Ghost', {}>
// | K<'Connections/POST', Fetch<api.Members.ConnectionsPOST>>
// | K<'Connections/DELETE', Fetch<api.Members.ConnectionsDELETE>>
// | K<'PushBreadcrumb', {id: string; title: string; isAdmin?: boolean}>
// | K<'PopBreadcrumb', {id: string}>
// | K<'Nav/SetOpen', {isOpen: boolean}>
// | K<'Nav/ToggleOpen', {}>
// | K<'ChoosePassword/UserPOST', Fetch<api.Auth.UserPOST>>
// | K<'ChoosePassword/WelcomeGET', Fetch<api.Auth.WelcomeGET>>
// | K<'PasswordReset/POST', Fetch<api.Auth.PasswordResetPOST>>
// | K<'Profile/GET', {slug: string; fetch: Fetch<api.Self.ProfileGET | api.Members.ProfileGET>}>
// | K<'ContactProfile/Other/GET', Fetch<api.Members.SlugGET>>
// | K<'AvatarModal/Reset', {}>
// | K<'AvatarModal/OnReceiveFile', {file: File; url: string}>
// | K<'AvatarModal/OnCropChange', Area>
// | K<'AvatarModal/OnImageLoaded', ImageSize>
