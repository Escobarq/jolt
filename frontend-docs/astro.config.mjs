// @ts-check
import { defineConfig } from 'astro/config';

const base = process.env.SITE_BASE ?? '/';
const site = process.env.SITE_URL ?? 'http://localhost:4321';
const configuredHosts = (process.env.DEV_ALLOWED_HOSTS ?? '')
	.split(',')
	.map((host) => host.trim())
	.filter(Boolean);
const allowedHosts = configuredHosts.length > 0 ? configuredHosts : ['.loca.lt'];

export default defineConfig({
	site,
	base: base === '/' ? undefined : base.replace(/\/+$/, ''),
	vite: {
		server: {
			allowedHosts,
		},
	},
});
