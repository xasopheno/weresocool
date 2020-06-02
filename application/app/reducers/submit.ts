// import { useContext } from 'react';
// import { DispatchContext } from '../actions/actions';
// import { GlobalContext } from '../store';
// import axios from 'axios';

// export const handleSubmit = async () => {
// const store = useContext(GlobalContext);
// const dispatch = useContext(DispatchContext);
// dispatch.onIncrementEditorType();

// let response = await axios.post('localhost:4588/api', {
// language: store.language
// });
// };
// const handleSubmit = async (new_language: string, save: boolean) => {
// try {
// setState(State.Rendering);
// let response = await axios.post(BACKEND_RENDER_URL, {
// language: new_language
// });
// if (renderSpace) {
// if (save) {
// localStorage.setItem('language', new_language);
// }
// let responseType = typeFromResponse(response.data);
// let value = valueFromResponse(response.data);
// console.log(responseType);
// console.log(value);
// switch (responseType) {
// case ResponseType.RenderSuccess:
// setResponseState(responseType);
// setErrorMessage('');
// break;
// case ResponseType.ParseError:
// const n_lines = new_language.split('\n').length;
// displayError(value, n_lines, renderSpace, setMarkers);
// setResponseState(responseType);
// setErrorMessage(`Line: ${value.line} | Column ${value.column}`);
// break;
// case ResponseType.IdError:
// setResponseState(responseType);
// setErrorMessage(`${value.id}`);
// break;
// case ResponseType.IndexError:
// setResponseState(responseType);
// setErrorMessage(value.message);
// break;
// case ResponseType.MsgError:
// console.log(value);
// //setResponseState(responseType);
// setErrorMessage('Error');
// break;
// default:
// console.log('Unhandled error');
// console.log(response);
// break;
// }
// }
// } catch (err) {
// console.log(err);
// }
// };
