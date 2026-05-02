import type { Metadata } from "next";
import Link from "next/link";
import { Inter, JetBrains_Mono } from "next/font/google";
import ThemeToggle from "@/components/theme-toggle";
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
  title: "NodeUnion Control Dashboard",
  description:
    "Dark-mode control dashboard for the NodeUnion decentralized compute marketplace.",
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
        <header className="app-header sticky top-0 z-50">
          <div className="mx-auto flex w-full max-w-7xl items-center justify-between px-4 py-3 sm:px-6 lg:px-8">
            <div>
              <Link href="/" className="brand-title font-mono text-xs tracking-[0.28em]">
                NODEUNION
              </Link>
              <p className="brand-subtitle mt-1 text-[11px] uppercase tracking-[0.32em]">
                Decentralized compute marketplace
              </p>
            </div>
            <nav className="flex items-center gap-2 text-sm">
              <Link href="/" className="nav-link">
                Landing
              </Link>
              <Link href="/networks" className="nav-link">
                Networks
              </Link>
              <Link href="/portfolio" className="nav-link">
                Portfolio
              </Link>
              <Link href="/provider" className="nav-link">
                Provider
              </Link>
              <Link href="/docs" className="nav-link">
                Docs
              </Link>
              <ThemeToggle />
            </nav>
          </div>
        </header>
        <div className="flex-1">{children}</div>
      </body>
    </html>
  );
}
