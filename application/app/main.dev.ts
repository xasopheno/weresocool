/* eslint-disable */

/**
 * This module executes inside of electron's main process. You can start
 * electron renderer process from here and communicate with the other processes
 * through IPC.
 *
 * When running `yarn build` or `yarn build-main`, this file is compiled to
 * `./app/main.prod.js` using webpack. This gives us some performance wins.
 */
import path from 'path';
import { app, BrowserWindow } from 'electron';
import { autoUpdater } from 'electron-updater';
import log from 'electron-log';
import MenuBuilder from './menu';
import child_process from 'child_process';
import getPort from 'get-port';

log.transports.file.level = 'info';
autoUpdater.logger = log;
log.info('App starting...');

const extraResourcesPath =
  process.env.NODE_ENV === 'development'
    ? path.join(path.dirname(__dirname) + '/extraResources')
    : path.join(process.resourcesPath + '/extraResources');

let mainWindow: BrowserWindow | null = null;

if (process.env.NODE_ENV === 'production') {
  const sourceMapSupport = require('source-map-support');
  sourceMapSupport.install();
}

if (
  process.env.NODE_ENV === 'development' ||
  process.env.DEBUG_PROD === 'true'
) {
  require('electron-debug')();
}

const installExtensions = async () => {
  const installer = require('electron-devtools-installer');
  const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
  const extensions = ['REACT_DEVELOPER_TOOLS'];

  return Promise.all(
    extensions.map((name) => installer.default(installer[name], forceDownload))
  ).catch(console.log);
};

const createWindow = async () => {
  if (
    process.env.NODE_ENV === 'development' ||
    process.env.DEBUG_PROD === 'true'
  ) {
    await installExtensions();
  }

  let server_path = path.join(extraResourcesPath, 'weresocool_server');

  const port = await getPort({ port: 4588 });
  process.env.BACKEND_PORT = port.toString();

  let server = child_process.spawn(`PORT=${port} ` + server_path, [], {
    shell: true,
    stdio: 'inherit',
  });

  const showDevTools = process.env.NODE_ENV === 'development' ? true : false;

  mainWindow = new BrowserWindow({
    show: true,
    width: 1200,
    height: 1100,
    webPreferences: {
      devTools: showDevTools,
      nodeIntegration: true,
    },
  });
  mainWindow.setBackgroundColor('#454343');
  mainWindow.loadURL(`file://${__dirname}/app.html`);

  // @TODO: Use 'ready-to-show' event
  //        https://github.com/electron/electron/blob/master/docs/api/browser-window.md#using-ready-to-show-event
  mainWindow.webContents.on('did-finish-load', () => {
    if (!mainWindow) {
      throw new Error('"mainWindow" is not defined');
    }
    if (process.env.START_MINIMIZED) {
      mainWindow.minimize();
    } else {
      mainWindow.show();
      mainWindow.focus();
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
    server.kill('SIGKILL');
    console.log('Server Stopped');
    app.quit();
  });

  const menuBuilder = new MenuBuilder(mainWindow);
  menuBuilder.buildMenu();

  autoUpdater.checkForUpdatesAndNotify();
};

/**
 * Add event listeners...
 */
// autoUpdater.logger = require("electron-log");
// autoUpdater.logger.transports.file.level = "info";

// autoUpdater.on('update-downloaded', () => {
// console.log('update-downloaded lats quitAndInstall');

// if (process.env.NODE_ENV === 'production') {
// dialog.showMessageBox({
// type: 'info',
// title: 'Found Updates',
// message: 'Found updates, do you want update now?',
// buttons: ['Sure', 'No']
// }, (buttonIndex) => {
// if (buttonIndex === 0) {
// const isSilent = true;
// const isForceRunAfter = true;
// autoUpdater.quitAndInstall(isSilent, isForceRunAfter);
// }
// else {
// updater.enabled = true
// updater = null
// }
// })
// }

// })

app.on('window-all-closed', () => {
  // Respect the OSX convention of having the application in memory even
  // after all windows have been closed
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('ready', createWindow);

app.on('activate', () => {
  // On macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (mainWindow === null) createWindow();
});
