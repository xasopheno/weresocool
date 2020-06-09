/* eslint-disable */
// See: https://medium.com/@TwitterArchiveEraser/notarize-electron-apps-7a5f988406db

const fs = require('fs');
const path = require('path');
var electron_notarize = require('electron-notarize');

module.exports = async function (params) {
  const notarize = process.env.NOTARIZE;
  // Only notarize the app on Mac OS only.
  if (notarize !== 'true' || process.platform !== 'darwin') {
    console.log('App not notorized');
    return;
  }
  console.log('afterSign hook triggered', params);

  // Same appId in electron-builder.
  let appId = 'org.xasopheno.WereSoCool';

  let appPath = path.join(
    params.appOutDir,
    `${params.packager.appInfo.productFilename}.app`
  );
  if (!fs.existsSync(appPath)) {
    throw new Error(`Cannot find application at: ${appPath}`);
  }

  console.log(`Notarizing ${appId} found at ${appPath}`);
  const appleId = 'danny.meyer@gmail.com';
  const appleIdPassword = `@keychain:AC_PASSWORD`;
  try {
    await electron_notarize.notarize({
      appBundleId: appId,
      appPath: appPath,
      //  appleId: process.env.appleId,
      appleId,
      //  appleIdPassword: process.env.appleIdPassword,
      appleIdPassword,
      ascProvider: 'FR76XXEQ7K',
    });
  } catch (error) {
    console.error(error);
  }

  console.log(`Done notarizing ${appId}`);
};
