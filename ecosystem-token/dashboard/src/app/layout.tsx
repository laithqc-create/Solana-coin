import type { Metadata } from 'next';
import { WalletContextProvider } from '@/lib/wallet';
import './globals.css';

export const metadata: Metadata = {
  title: 'Ecosystem Token Dashboard',
  description: 'Solana collateral-backed ecosystem token management',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <WalletContextProvider>
          <div className="min-h-screen bg-slate-950">
            {/* Navigation Header */}
            <header className="border-b border-slate-700 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
              <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex justify-between items-center">
                <h1 className="text-xl font-bold text-blue-400">
                  Ecosystem Token
                </h1>
                <nav className="flex gap-6">
                  <a
                    href="/"
                    className="text-slate-300 hover:text-blue-400 transition-colors"
                  >
                    Dashboard
                  </a>
                  <a
                    href="/treasury"
                    className="text-slate-300 hover:text-blue-400 transition-colors"
                  >
                    Treasury
                  </a>
                  <a
                    href="/analytics"
                    className="text-slate-300 hover:text-blue-400 transition-colors"
                  >
                    Analytics
                  </a>
                  <a
                    href="/admin"
                    className="text-slate-300 hover:text-blue-400 transition-colors"
                  >
                    Admin
                  </a>
                </nav>
              </div>
            </header>

            {/* Main Content */}
            <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
              {children}
            </main>

            {/* Footer */}
            <footer className="border-t border-slate-700 bg-slate-900/50 mt-16 py-8">
              <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-slate-400 text-sm">
                <p>Solana Ecosystem Token | Powered by Anchor</p>
              </div>
            </footer>
          </div>
        </WalletContextProvider>
      </body>
    </html>
  );
}
