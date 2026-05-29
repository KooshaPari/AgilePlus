import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
export default defineConfig({
    plugins: [react()],
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
            '@/components': path.resolve(__dirname, './src/components'),
            '@/lib': path.resolve(__dirname, './src/lib'),
            '@/pages': path.resolve(__dirname, './src/pages'),
        },
    },
    server: {
        port: 5173,
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:4000',
                changeOrigin: true,
            },
        },
    },
});
