/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Docker-inspired dark theme
        docker: {
          dark: '#0C1117',
          darker: '#010409',
          border: '#30363D',
          hover: '#161B22',
          accent: '#58A6FF',
          accentLight: '#79C0FF',
          success: '#3FB950',
          warning: '#D29922',
          danger: '#F85149',
        }
      },
      backgroundColor: {
        'docker-dark': '#0C1117',
        'docker-darker': '#010409',
        'docker-hover': '#161B22',
      }
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
