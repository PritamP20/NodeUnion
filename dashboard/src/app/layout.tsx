import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { AppShell } from "@/components/app-shell";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono",
});

export const metadata: Metadata = {
  title: "NodeUnion Dashboard",
  description:
    "A polished web control plane for the NodeUnion decentralized compute marketplace.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const themeInitScript = `
    (function () {
      try {
        var stored = localStorage.getItem("nodeunion-theme");
        var preferred = window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
        var theme = stored === "light" || stored === "dark" ? stored : preferred;
        document.documentElement.setAttribute("data-theme", theme);
        document.documentElement.style.colorScheme = theme;
      } catch (e) {
        document.documentElement.setAttribute("data-theme", "dark");
        document.documentElement.style.colorScheme = "dark";
      }
    })();
  `;

  return (
    <html lang="en" className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}>
      <body className="app-shell min-h-full flex flex-col text-slate-100">
        <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
        <AppShell>{children}</AppShell>
      </body>
    </html>
  );
}
