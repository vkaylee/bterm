import { test as base, expect, request } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';

export { expect };

interface WorkerFixtures {
  server: {
    url: string;
    process: ChildProcess;
  };
}

export const test = base.extend<{}, WorkerFixtures>({
  server: [async ({}, use, testInfo) => {
    console.log('Starting worker backend server...');
    const projectRoot = path.resolve(__dirname, '..');
    const noAutoLogin = testInfo.title.includes('@noAutoLogin');
    
    const env: Record<string, string> = { 
      ...process.env, 
      PORT: '0',
      DATABASE_URL: 'sqlite::memory:',
    };

    if (!noAutoLogin) {
      env.SKIP_ADMIN_PWD_CHANGE = '1';
    }

    // Start backend with PORT=0 (auto-assign) and in-memory DB for isolation
    const serverProcess = spawn('cargo', ['run'], {
      cwd: projectRoot,
      env,
      stdio: ['ignore', 'pipe', 'inherit'],
    });

    let serverUrl = '';
    const readyPromise = new Promise<string>((resolve, reject) => {
      serverProcess.stdout?.on('data', (data) => {
        const output = data.toString();
        // Look for: 🚀 BTerminal is running on http://localhost:XXXXX
        const match = output.match(/http:\/\/localhost:\d+/);
        if (match) {
          serverUrl = match[0];
          console.log(`Worker server ready at: ${serverUrl}`);
          resolve(serverUrl);
        }
      });

      serverProcess.on('error', (err) => {
        reject(new Error(`Failed to start server: ${err.message}`));
      });

      serverProcess.on('exit', (code) => {
        if (!serverUrl) {
          reject(new Error(`Server exited prematurely with code ${code}`));
        }
      });
    });

    try {
      const url = await readyPromise;
      await use({ url, process: serverProcess });
    } finally {
      console.log(`Shutting down worker server at ${serverUrl}...`);
      serverProcess.kill();
    }
  }, { scope: 'test', auto: true }],

  baseURL: async ({ server }, use, testInfo) => {
    let url = server.url;
    if (testInfo.project.name === 'DOM-Fallback') {
      url += '?renderer=dom';
    } else if (testInfo.project.name === 'Canvas') {
      url += '?renderer=canvas';
    } else if (testInfo.project.name === 'WebGL') {
      url += '?renderer=webgl';
    }
    await use(url);
  },

  // Automatically login for each context
  context: async ({ context, server }, use, testInfo) => {
    const noAutoLogin = testInfo.title.includes('@noAutoLogin');
    
    if (!noAutoLogin) {
      // Perform login via API
      const apiContext = await request.newContext({ baseURL: server.url });
      const loginResponse = await apiContext.post('/api/auth/login', {
        data: {
          username: 'admin',
          password: 'admin'
        }
      });

      if (!loginResponse.ok()) {
        throw new Error(`Auto-login failed: ${loginResponse.status()} ${loginResponse.statusText()}`);
      }

      // Transfer cookies to the context
      const cookies = await apiContext.storageState();
      await context.addCookies(cookies.cookies);
    }
    
    await use(context);
  },
});
