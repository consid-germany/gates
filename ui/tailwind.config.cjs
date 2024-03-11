/** @type {import('tailwindcss').Config}*/
const config = {
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'
	],
	theme: {
		fontFamily: {
			sans: ['Plus Jakarta Sans Variable', 'sans-serif'],
		},
		extend: {
			colors: {
				primary: {
					50: '#f8fafc',
					100: '#f1f5f9',
					200: '#e2e8f0',
					300: '#cbd5e1',
					400: '#94a3b8',
					500: '#64748b',
					600: '#475569',
					700: '#334155',
					800: '#1e293b',
					900: '#0f172a'
				}
			}
		}
	},
	safelist: [
		{
			pattern: /grid-cols-./,
			variants: ['md']
		}
	],
	plugins: [require('flowbite/plugin')],
	darkMode: 'class'
};

module.exports = config;
