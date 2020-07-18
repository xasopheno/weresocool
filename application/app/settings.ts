console.log('BACKEND_PORT:', process.env.BACKEND_PORT);

export const settings = {
  backendURL: `http://localhost:${process.env.BACKEND_PORT}/api/render`,
  printURL: `http://localhost:${process.env.BACKEND_PORT}/api/print`,
};
