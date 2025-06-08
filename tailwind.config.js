module.exports = {
    mode: "jit",
    content: {
        files: ["src/**/*.rs", "index.html"],
    },
    darkMode: "media", // 'media' or 'class'
    theme: {
        extend: {
            colors: {
                'tomato-red': '#e74c3c',
                'tomato-green': '#27ae60',
                'tomato-orange': '#e67e22',
                'dark-tomato': '#c0392b',
                'light-tomato': '#fadbd8',
            }
        },
    },
    variants: {
        extend: {},
    },
    plugins: [],
};