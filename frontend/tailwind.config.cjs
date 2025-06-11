/** @type {import('tailwindcss').Config} */
module.exports = {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				dark: '#191825',
				yellow: '#f1d00a',
				blue: '#1b56fd'
			},
			fontFamily: {
				excon: ['Excon', 'sans-serif']
			},
			letterSpacing: {
				wider: '0.5px'
			},
			scrollbar: {
				none: 'none'
			}
		}
	},
	plugins: []
};
