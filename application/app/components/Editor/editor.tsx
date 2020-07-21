import React, { useState, useEffect, useContext } from 'react';
import AceEditor from 'react-ace';
import WSCMode from './mode.js';
import './theme';
import { GlobalContext, Editors } from '../../store';
import { DispatchContext } from '../../actions/actions';

import 'ace-builds/src-noconflict/mode-elixir';
import 'ace-builds/src-noconflict/theme-github';
import 'ace-builds/src-noconflict/keybinding-vim';
import 'ace-builds/src-noconflict/keybinding-emacs';
import 'ace-builds/src-noconflict/ext-language_tools';

const customMode = new WSCMode();

type Props = { handleLoad: () => void };
function debounce(fn, ms) {
  let timer;
  return (_) => {
    clearTimeout(timer);
    timer = setTimeout((_) => {
      timer = null;
      fn.apply(this, arguments);
    }, ms);
  };
}

export const Editor = (props: Props) => {
  const [dimensions, setDimensions] = React.useState({
    height: window.innerHeight,
    width: window.innerWidth,
  });
  React.useEffect(() => {
    const debouncedHandleResize = debounce(function handleResize() {
      setDimensions({
        height: window.innerHeight,
        width: window.innerWidth,
      });
    }, 300);

    window.addEventListener('resize', debouncedHandleResize);

    return (_) => {
      window.removeEventListener('resize', debouncedHandleResize);
    };
  });
  // return (
  // <div>
  // Rendered at {dimensions.width} x {dimensions.height}
  // </div>
  // );
  // };

  // export const Editor2 = (props: Props): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);

  const [renderSpace, setRenderSpace] = useState<AceEditor | null>();
  const [render, setRender] = useState<boolean>(false);
  const [save, setSave] = useState<boolean>(false);

  useEffect(() => {
    if (renderSpace) {
      const getLocalStorage = () => {
        const storedLanguage = localStorage.getItem('language');
        const storedEditor = localStorage.getItem('editor');
        if (storedLanguage) {
          dispatch.onUpdateLanguage(storedLanguage);
        }
        if (storedEditor) {
          dispatch.onIncrementEditorType(parseInt(storedEditor) - 1);
        }
      };
      // @ts-ignore
      renderSpace.editor.getSession().setMode(customMode);
      renderSpace.editor.setTheme('ace/theme/wsc');
      getLocalStorage();
    }
  }, [renderSpace, dispatch]);

  useEffect(() => {
    const submit = async () => {
      if (render) {
        if (renderSpace) {
          renderSpace.editor.resize();
        }
        await dispatch.onRender(store.language, store.volume);
        setRender(false);
      }
    };

    submit().catch((e) => {
      throw e;
    });
  }, [render, dispatch, store.language, store.volume, renderSpace]);

  useEffect(() => {
    if (save) {
      dispatch.onFileSave(store.language);
      setSave(false);
    }
  }, [store, dispatch, store.language, save]);

  useEffect(() => {
    if (store.initializeTest) {
      dispatch.onStop().catch((e) => {
        throw e;
      });
    }
  }, [dispatch, store.initializeTest]);

  const setRenderSpaceOuter = (el: AceEditor | null) => {
    if (el && !store.editor_ref) {
      dispatch.onSetEditorRef(el);
      setRenderSpace(el);
    }
  };

  let windowScale = window.innerHeight > 700 ? 0.8 : 0.7;
  windowScale = window.innerHeight < 500 ? 0.6 : windowScale;
  windowScale = window.innerHeight < 375 ? 0.5 : windowScale;
  return (
    <div
      style={{
        height: window.innerHeight * windowScale,
      }}
    >
      <AceEditor
        style={{
          height: window.innerHeight * windowScale,
          width: '80vw',
          marginLeft: '10vw',
        }}
        focus={true}
        ref={(el) => {
          setRenderSpaceOuter(el);
        }}
        placeholder="WereSoCool"
        mode="elixir"
        theme="github"
        name="aceEditor"
        keyboardHandler={Editors[store.editor]['style']}
        value={store.language}
        onChange={(l) => dispatch.onUpdateLanguage(l)}
        markers={store.markers}
        fontSize={20}
        showPrintMargin={true}
        showGutter={true}
        highlightActiveLine={true}
        setOptions={{
          //enableLiveAutocompletion: true,
          enableBasicAutocompletion: true,
          showLineNumbers: true,
          tabSize: 2,
          displayIndentGuides: true,
        }}
        commands={[
          {
            name: 'submit',
            bindKey: { win: 'Shift-Enter', mac: 'Shift-Enter' },
            exec: () => {
              setRender(true);
            },
          },
          {
            name: 'stop',
            bindKey: { win: 'Command-p', mac: 'Command-Enter' },
            exec: async () => {
              await dispatch.onStop();
            },
          },
          {
            name: 'save',
            bindKey: { win: 'Ctrl-s', mac: 'Command-s' },
            exec: () => {
              setSave(true);
            },
          },
          {
            name: 'load',
            bindKey: { win: 'Ctrl-l', mac: 'Command-l' },
            exec: () => {
              props.handleLoad();
            },
          },
        ]}
      />
    </div>
  );
};
