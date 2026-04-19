import type { Metadata } from "next";
import Link from "next/link";
import { Inter, JetBrains_Mono } from "next/font/google";
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
  return (
    <html lang="en" className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}>
      <body className="min-h-full flex flex-col bg-nodeunion-shell text-slate-100">
        <header className="sticky top-0 z-50 border-b border-white/5 bg-[#0d1117]/75 backdrop-blur-xl">
          <div className="mx-auto flex w-full max-w-7xl items-center justify-between px-4 py-3 sm:px-6 lg:px-8">
            <div>
              <Link href="/" className="font-mono text-xs tracking-[0.28em] text-sky-300/90">
                NODEUNION
              </Link>
              <p className="mt-1 text-[11px] uppercase tracking-[0.32em] text-slate-400">
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
            </nav>
          </div>
        </header>
        <div className="flex-1">{children}</div>
      </body>
    </html>
  );
}
