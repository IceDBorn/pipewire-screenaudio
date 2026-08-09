import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
	plugins: [react()],
	base: "/react/dist",
	css: {
		modules: {
			localsConvention: "camelCaseOnly",
		},
	},
	build: {
		sourcemap: "inline",
		chunkSizeWarningLimit: Infinity,
	},
});
