import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";

export const metadata: Metadata = {
  title: "NodeUnion Control Dashboard",
  description:
    "Landing, provider operations, user portfolio, and complete docs for the NodeUnion compute exchange.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased">
      <body className="min-h-full flex flex-col">
        <header className="sticky top-0 z-50 border-b border-cyan-900/30 bg-slate-950/60 backdrop-blur">
          <div className="mx-auto flex w-full max-w-7xl items-center justify-between px-4 py-3 sm:px-6 lg:px-8">
            <Link href="/" className="font-mono text-xs tracking-[0.22em] text-cyan-200/90">
              NODEUNION
            </Link>
            <nav className="flex items-center gap-2 text-sm">
              <Link href="/" className="nav-link">
                Landing
              </Link>
              <Link href="/provider" className="nav-link">
                Provider + Deploy
              </Link>
              <Link href="/portfolio" className="nav-link">
                Portfolio
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
