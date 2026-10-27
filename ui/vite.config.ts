import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const apiTarget = process.env.COMMAND_SCHEDULER_API_URL ?? 'http://127.0.0.1:30002'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [{ name: 'api-libraries', test: /node_modules/, minSize: 10000, maxSize: 250000 }],
        },
      },
    },
  },
  server: {
    host: '127.0.0.1',
    port: Number(process.env.COMMAND_SCHEDULER_WEB_PORT ?? 30014),
    strictPort: true,
    proxy: {
      '/api': {
        target: apiTarget,
        configure: (proxy) => {
          proxy.on('proxyReq', (proxyRequest, request) => {
            if (request.headers.host) proxyRequest.setHeader('host', request.headers.host)
          })
        },
      },
      '/health': {
        target: apiTarget,
        configure: (proxy) => {
          proxy.on('proxyReq', (proxyRequest, request) => {
            if (request.headers.host) proxyRequest.setHeader('host', request.headers.host)
          })
        },
      },
    },
  },
})
