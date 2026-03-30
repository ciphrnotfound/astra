import type { Metadata } from "next";
import { DM_Sans, Space_Grotesk } from "next/font/google";
import localFont from "next/font/local";
import "./globals.css";

const dmSans = DM_Sans({
  variable: "--font-dm-sans",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const spaceGrotesk = Space_Grotesk({
  variable: "--font-space-grotesk",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const cabinetGrotesk = localFont({
  src: [
    {
      path: "../public/CabinetGrotesk_Complete/Fonts/WEB/fonts/CabinetGrotesk-Regular.woff2",
      weight: "400",
      style: "normal",
    },
    {
      path: "../public/CabinetGrotesk_Complete/Fonts/WEB/fonts/CabinetGrotesk-Medium.woff2",
      weight: "500",
      style: "normal",
    },
  ],
  variable: "--font-cabinet-grotesk",
});

export const metadata: Metadata = {
  title: "Astra - Code That Understands Itself",
  description: "AI-powered CLI that understands your entire codebase. Cross-language migrations, time travel debugging, and semantic refactoring.",
  icons: {
    icon: [
      
      { url: '/favicon2/favicon-96x96.png', sizes: '96x96', type: 'image/png' },
      { url: '/favicon2/favicon.ico', sizes: 'any' },
    ],
    shortcut: '/favicon2/favicon.ico',
    apple: [
      { url: '/favicon2/apple-icon-57x57.png', sizes: '57x57', type: 'image/png' },
      { url: '/favicon2/apple-icon-60x60.png', sizes: '60x60', type: 'image/png' },
      { url: '/favicon2/apple-icon-72x72.png', sizes: '72x72', type: 'image/png' },
      { url: '/favicon2/apple-icon-76x76.png', sizes: '76x76', type: 'image/png' },
      { url: '/favicon2/apple-icon-114x114.png', sizes: '114x114', type: 'image/png' },
      { url: '/favicon2/apple-icon-120x120.png', sizes: '120x120', type: 'image/png' },
      { url: '/favicon2/apple-icon-144x144.png', sizes: '144x144', type: 'image/png' },
      { url: '/favicon2/apple-icon-152x152.png', sizes: '152x152', type: 'image/png' },
      { url: '/favicon2/apple-icon-180x180.png', sizes: '180x180', type: 'image/png' },
    ],
    other: [
      { rel: 'icon', url: '/favicon2/android-icon-192x192.png', sizes: '192x192', type: 'image/png' },
    ],
  },
  manifest: '/favicon2/manifest.json',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${dmSans.variable} ${cabinetGrotesk.variable} ${spaceGrotesk.variable}`}>
      <body className={dmSans.className}>{children}</body>
    </html>
  );
}
